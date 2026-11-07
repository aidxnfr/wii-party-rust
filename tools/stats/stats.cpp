// Claude OPT: live performance stats window for Wii Party Rust. stats.exe <runtime.log>
// stats.exe <runtime.log> [game pid]. Reads per-thread CPU, memory of the game (the pid the launcher passes, else
// wii-party.exe / wii-party-hybrid.exe by name) and tails the game's runtime.log once per 500 ms. No admin needed.
#define UNICODE
#include <windows.h>
#include <psapi.h>
#include <tlhelp32.h>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>
#include <map>
#include <algorithm>
static std::wstring g_log; static HWND g_wnd; static DWORD g_pid = 0; static bool g_seen = false; static int g_gone = 0;
static ULONGLONG g_lastWall = 0, g_lastProc = 0; static std::map<DWORD, ULONGLONG> g_lastThread;
static std::vector<std::wstring> g_lines; static std::vector<double> g_fps; static double g_procPct = 0; static SIZE_T g_priv = 0, g_ws = 0;
// Smooth motion (display-rate interpolation) switch: writes 0..3 to interp.txt next to the log; the game checks it every half second.
static std::wstring g_interpPath; static int g_interp = 0; static RECT g_btn[4];
static void loadInterp() { FILE* f = _wfopen(g_interpPath.c_str(), L"rb"); if (!f) return; char b[16] = {0}; fread(b, 1, 15, f); fclose(f); g_interp = atoi(b); if (g_interp < 0 || g_interp > 3) g_interp = 0; }
static void saveInterp(int v) { g_interp = v; FILE* f = _wfopen(g_interpPath.c_str(), L"wb"); if (f) { fprintf(f, "%d\n", v); fclose(f); } }
struct T { DWORD id; double pct; std::wstring name; }; static std::vector<T> g_threads;
typedef HRESULT (WINAPI *GTD)(HANDLE, PWSTR*);
static ULONGLONG ft(const FILETIME& f) { return ((ULONGLONG)f.dwHighDateTime << 32) | f.dwLowDateTime; }
static DWORD g_given = 0;  // pid from the launcher; used while that process is alive
static DWORD findPid() {
    if (g_given) { HANDLE h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, g_given); DWORD code = 0;
        bool alive = h && GetExitCodeProcess(h, &code) && code == STILL_ACTIVE; if (h) CloseHandle(h); if (alive) return g_given; g_given = 0; }
    HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0); PROCESSENTRY32W e; e.dwSize = sizeof e; DWORD pid = 0;
    if (s != INVALID_HANDLE_VALUE && Process32FirstW(s, &e)) do { if (!_wcsicmp(e.szExeFile, L"wii-party.exe") || !_wcsicmp(e.szExeFile, L"wii-party-hybrid.exe")) pid = e.th32ProcessID; } while (Process32NextW(s, &e));
    if (s != INVALID_HANDLE_VALUE) CloseHandle(s); return pid;
}
static std::wstring widen(const std::string& s) { int n = MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), nullptr, 0); std::wstring w(n, 0); MultiByteToWideChar(CP_UTF8, 0, s.c_str(), (int)s.size(), &w[0], n); return w; }
static void tailLog() {
    FILE* f = _wfopen(g_log.c_str(), L"rb"); if (!f) return;
    _fseeki64(f, 0, SEEK_END); long long size = _ftelli64(f); long long start = size > 65536 ? size - 65536 : 0; _fseeki64(f, start, SEEK_SET);
    std::string buf((size_t)(size - start), 0); size_t got = fread(&buf[0], 1, buf.size(), f); fclose(f); buf.resize(got);
    const char* keys[] = {"] fps", "] threads", "] copies", "] output", "] dsp", "] memory", "] audio"};
    std::vector<std::string> found(7); std::vector<double> fps; int slow = 0;
    size_t pos = 0;
    while (pos < buf.size()) {
        size_t e = buf.find('\n', pos); if (e == std::string::npos) e = buf.size();
        std::string line = buf.substr(pos, e - pos); pos = e + 1;
        if (!line.empty() && line.back() == '\r') line.pop_back();
        for (int k = 0; k < 7; ++k) if (line.find(keys[k]) != std::string::npos) found[k] = line;
        if (line.find("] fps") != std::string::npos) { size_t q = line.find("fps"); fps.push_back(atof(line.c_str() + q + 3)); }
        if (line.find("slow frame") != std::string::npos) ++slow;
    }
    g_lines.clear();
    for (int k = 0; k < 7; ++k) if (!found[k].empty()) { size_t b = found[k].find(']'); g_lines.push_back(widen(b == std::string::npos ? found[k] : found[k].substr(b + 2))); }
    wchar_t t[96]; swprintf(t, 96, L"slow frames (last 64 KB of log): %d", slow); g_lines.push_back(t);
    if (fps.size() > 120) fps.erase(fps.begin(), fps.end() - 120); g_fps = fps;
}
static void sample() {
    if (!g_pid || !g_seen) { g_pid = findPid(); if (g_pid) { g_seen = true; g_lastThread.clear(); g_lastProc = 0; } }
    HANDLE p = g_pid ? OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, FALSE, g_pid) : nullptr;
    DWORD code = 0; if (p && GetExitCodeProcess(p, &code) && code != STILL_ACTIVE) { CloseHandle(p); p = nullptr; }
    if (!p) { if (g_seen && ++g_gone > 6) PostQuitMessage(0); g_pid = 0; g_seen = false; tailLog(); return; }
    g_gone = 0;
    FILETIME now; GetSystemTimeAsFileTime(&now); ULONGLONG wall = ft(now);
    FILETIME c, x, k, u; GetProcessTimes(p, &c, &x, &k, &u); ULONGLONG proc = ft(k) + ft(u);
    double dw = g_lastWall ? (double)(wall - g_lastWall) : 0;
    if (dw > 0 && g_lastProc) g_procPct = 100.0 * (proc - g_lastProc) / dw;
    PROCESS_MEMORY_COUNTERS_EX m; m.cb = sizeof m; if (GetProcessMemoryInfo(p, (PROCESS_MEMORY_COUNTERS*)&m, sizeof m)) { g_priv = m.PrivateUsage; g_ws = m.WorkingSetSize; }
    static GTD gtd = (GTD)GetProcAddress(GetModuleHandleW(L"kernel32.dll"), "GetThreadDescription");
    std::vector<T> ts; std::map<DWORD, ULONGLONG> cur;
    HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0); THREADENTRY32 te; te.dwSize = sizeof te;
    if (s != INVALID_HANDLE_VALUE && Thread32First(s, &te)) do {
        if (te.th32OwnerProcessID != g_pid) continue;
        HANDLE h = OpenThread(THREAD_QUERY_LIMITED_INFORMATION, FALSE, te.th32ThreadID); if (!h) continue;
        FILETIME a, b, kk, uu; if (GetThreadTimes(h, &a, &b, &kk, &uu)) {
            ULONGLONG v = ft(kk) + ft(uu); cur[te.th32ThreadID] = v;
            auto it = g_lastThread.find(te.th32ThreadID); double pct = (it != g_lastThread.end() && dw > 0) ? 100.0 * (v - it->second) / dw : 0;
            std::wstring name; PWSTR d = nullptr; if (gtd && SUCCEEDED(gtd(h, &d)) && d) { name = d; LocalFree(d); }
            ts.push_back({te.th32ThreadID, pct, name});
        }
        CloseHandle(h);
    } while (Thread32Next(s, &te));
    if (s != INVALID_HANDLE_VALUE) CloseHandle(s);
    std::sort(ts.begin(), ts.end(), [](const T& a, const T& b) { return a.pct > b.pct; }); if (ts.size() > 7) ts.resize(7);
    g_threads = ts; g_lastThread = cur; g_lastWall = wall; g_lastProc = proc; CloseHandle(p);
    tailLog();
}
static void paint(HDC dc, RECT r) {
    HBRUSH bg = CreateSolidBrush(RGB(18, 20, 24)); FillRect(dc, &r, bg); DeleteObject(bg);
    SetBkMode(dc, TRANSPARENT);
    HFONT big = CreateFontW(-22, 0, 0, 0, FW_BOLD, 0, 0, 0, 0, 0, 0, CLEARTYPE_QUALITY, 0, L"Segoe UI");
    HFONT mono = CreateFontW(-15, 0, 0, 0, FW_NORMAL, 0, 0, 0, 0, 0, 0, CLEARTYPE_QUALITY, FIXED_PITCH, L"Consolas");
    int y = 10; wchar_t t[256];
    SelectObject(dc, big); SetTextColor(dc, RGB(235, 235, 240));
    double last = g_fps.empty() ? 0 : g_fps.back();
    if (g_pid) swprintf(t, 256, L"%.1f fps   CPU %.0f%% of a core   %llu MB", last, g_procPct, (unsigned long long)(g_priv >> 20)); else swprintf(t, 256, L"Waiting for the game...");
    TextOutW(dc, 12, y, t, (int)wcslen(t)); y += 34;
    {
        HFONT ui = CreateFontW(-15, 0, 0, 0, FW_SEMIBOLD, 0, 0, 0, 0, 0, 0, CLEARTYPE_QUALITY, 0, L"Segoe UI"); SelectObject(dc, ui);
        SetTextColor(dc, RGB(235, 235, 240)); TextOutW(dc, 12, y + 5, L"Smooth motion:", 14);
        const wchar_t* names[4] = {L"Off (60)", L"2x (120 Hz)", L"3x (180 Hz)", L"4x (240 Hz)"};
        int bx = 125;
        for (int i = 0; i < 4; ++i) {
            RECT b = {bx, y, bx + 96, y + 28}; g_btn[i] = b; bx += 102;
            HBRUSH br = CreateSolidBrush(i == g_interp ? RGB(60, 140, 90) : RGB(45, 50, 60)); FillRect(dc, &b, br); DeleteObject(br);
            SetTextColor(dc, RGB(240, 240, 245)); DrawTextW(dc, names[i], -1, &b, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        }
        DeleteObject(ui); y += 36;
        SelectObject(dc, mono); SetTextColor(dc, RGB(130, 140, 155));
        const wchar_t* note = L"Game logic stays at 60. Extra frames are blended in between (adds ~10 ms delay).";
        TextOutW(dc, 12, y, note, (int)wcslen(note)); y += 24;
    }
    // fps graph (last 120 s), 0..70 fps, 60 fps guide line
    int gx = 12, gw = r.right - 24, gh = 70, gy = y;
    HPEN guide = CreatePen(PS_DOT, 1, RGB(70, 75, 85)); SelectObject(dc, guide); MoveToEx(dc, gx, gy + gh - gh * 60 / 70, nullptr); LineTo(dc, gx + gw, gy + gh - gh * 60 / 70);
    HPEN line = CreatePen(PS_SOLID, 2, RGB(90, 200, 120)); SelectObject(dc, line);
    for (size_t i = 0; i < g_fps.size(); ++i) { double v = std::min(70.0, g_fps[i]); int px = gx + (int)(gw * (i + 120 - g_fps.size()) / 119.0); int py = gy + gh - (int)(gh * v / 70.0); if (i == 0) MoveToEx(dc, px, py, nullptr); else LineTo(dc, px, py); }
    DeleteObject(guide); DeleteObject(line); y += gh + 12;
    SelectObject(dc, mono); SetTextColor(dc, RGB(160, 170, 185));
    swprintf(t, 256, L"memory: private %llu MB, working set %llu MB", (unsigned long long)(g_priv >> 20), (unsigned long long)(g_ws >> 20)); TextOutW(dc, 12, y, t, (int)wcslen(t)); y += 22;
    SetTextColor(dc, RGB(235, 235, 240)); TextOutW(dc, 12, y, L"busiest threads (% of one core):", 32); y += 20; SetTextColor(dc, RGB(160, 170, 185));
    for (auto& th : g_threads) { swprintf(t, 256, L"  %5.1f%%  #%-6lu %ls", th.pct, (unsigned long)th.id, th.name.empty() ? L"" : th.name.c_str()); TextOutW(dc, 12, y, t, (int)wcslen(t)); y += 18; }
    y += 8; SetTextColor(dc, RGB(235, 235, 240)); TextOutW(dc, 12, y, L"game log (latest):", 18); y += 20; SetTextColor(dc, RGB(160, 170, 185));
    for (auto& l : g_lines) { RECT lr = {12, y, r.right - 8, y + 36}; DrawTextW(dc, l.c_str(), (int)l.size(), &lr, DT_WORDBREAK | DT_NOPREFIX); y += 36; }
    DeleteObject(big); DeleteObject(mono);
}
static LRESULT CALLBACK proc(HWND h, UINT msg, WPARAM w, LPARAM l) {
    switch (msg) {
    case WM_LBUTTONDOWN: { POINT pt = {(short)LOWORD(l), (short)HIWORD(l)}; for (int i = 0; i < 4; ++i) if (PtInRect(&g_btn[i], pt)) { saveInterp(i); InvalidateRect(h, nullptr, FALSE); } return 0; }
    case WM_TIMER: sample(); InvalidateRect(h, nullptr, FALSE); return 0;
    case WM_PAINT: { PAINTSTRUCT ps; HDC dc = BeginPaint(h, &ps); RECT r; GetClientRect(h, &r); HDC mem = CreateCompatibleDC(dc); HBITMAP bmp = CreateCompatibleBitmap(dc, r.right, r.bottom); SelectObject(mem, bmp); paint(mem, r); BitBlt(dc, 0, 0, r.right, r.bottom, mem, 0, 0, SRCCOPY); DeleteObject(bmp); DeleteDC(mem); EndPaint(h, &ps); return 0; }
    case WM_ERASEBKGND: return 1;
    case WM_DESTROY: PostQuitMessage(0); return 0;
    }
    return DefWindowProcW(h, msg, w, l);
}
int WINAPI wWinMain(HINSTANCE inst, HINSTANCE, PWSTR cmd, int show) {
    int argc; LPWSTR* argv = CommandLineToArgvW(GetCommandLineW(), &argc); if (argc > 1) g_log = argv[1]; if (argc > 2) g_given = (DWORD)_wtoi(argv[2]);
    { size_t s = g_log.find_last_of(L"\\/"); g_interpPath = (s == std::wstring::npos ? std::wstring() : g_log.substr(0, s + 1)) + L"interp.txt"; loadInterp(); }
    WNDCLASSW wc = {}; wc.lpfnWndProc = proc; wc.hInstance = inst; wc.lpszClassName = L"WiiPartyStats"; wc.hCursor = LoadCursor(nullptr, IDC_ARROW); RegisterClassW(&wc);
    g_wnd = CreateWindowExW(0, L"WiiPartyStats", L"Wii Party - Performance stats", WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT, 560, 680, nullptr, nullptr, inst, nullptr);
    ShowWindow(g_wnd, SW_SHOWNOACTIVATE); SetTimer(g_wnd, 1, 500, nullptr); sample();
    MSG m; while (GetMessageW(&m, nullptr, 0, 0)) { TranslateMessage(&m); DispatchMessageW(&m); }
    return 0;
}

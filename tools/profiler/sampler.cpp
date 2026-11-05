// Claude opt lane: no-admin flat sampling profiler. sampler.exe <pid> <seconds> <intervalus> <out.txt>
// Samples RIP of every thread of <pid> (SuspendThread + GetThreadContext), aggregates per thread, dumps module map.
#include <windows.h>
#include <tlhelp32.h>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <vector>
#include <unordered_map>
int main(int argc, char** argv) {
    if (argc < 5) { std::fprintf(stderr, "usage: sampler pid seconds intervalus out\n"); return 2; }
    DWORD pid = std::strtoul(argv[1], nullptr, 10); double secs = std::atof(argv[2]); int iv = std::atoi(argv[3]);
    FILE* out = std::fopen(argv[4], "w"); if (!out) return 3;
    std::map<DWORD, HANDLE> threads; std::map<DWORD, std::unordered_map<unsigned long long, unsigned>> hist; std::map<DWORD, unsigned> total;
    timeBeginPeriod(1);
    LARGE_INTEGER f, t0, t; QueryPerformanceFrequency(&f); QueryPerformanceCounter(&t0);
    unsigned ticks = 0;
    for (;;) {
        QueryPerformanceCounter(&t); if ((double)(t.QuadPart - t0.QuadPart) / f.QuadPart > secs) break;
        if (ticks % 200 == 0) {
            HANDLE s = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0); THREADENTRY32 te; te.dwSize = sizeof te;
            if (s != INVALID_HANDLE_VALUE && Thread32First(s, &te)) do {
                if (te.th32OwnerProcessID == pid && !threads.count(te.th32ThreadID)) {
                    HANDLE h = OpenThread(THREAD_SUSPEND_RESUME | THREAD_GET_CONTEXT | THREAD_QUERY_INFORMATION, FALSE, te.th32ThreadID);
                    if (h) threads[te.th32ThreadID] = h;
                }
            } while (Thread32Next(s, &te));
            if (s != INVALID_HANDLE_VALUE) CloseHandle(s);
        }
        for (auto& [tid, h] : threads) {
            if (SuspendThread(h) == (DWORD)-1) continue;
            CONTEXT c; c.ContextFlags = CONTEXT_CONTROL;
            if (GetThreadContext(h, &c)) { hist[tid][c.Rip]++; total[tid]++; }
            ResumeThread(h);
        }
        ++ticks;
        if (iv >= 1000) Sleep(iv / 1000); else { LARGE_INTEGER a, b; QueryPerformanceCounter(&a); do QueryPerformanceCounter(&b); while ((b.QuadPart - a.QuadPart) * 1000000 / f.QuadPart < iv); }
    }
    HANDLE ms = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid); MODULEENTRY32 me; me.dwSize = sizeof me;
    if (ms != INVALID_HANDLE_VALUE && Module32First(ms, &me)) do { std::fprintf(out, "MOD %llx %lx %s\n", (unsigned long long)me.modBaseAddr, (unsigned long)me.modBaseSize, me.szModule); } while (Module32Next(ms, &me));
    std::fprintf(out, "TICKS %u\n", ticks);
    for (auto& [tid, m] : hist) {
        FILETIME a, b, k, u; unsigned long long user = 0; if (GetThreadTimes(threads[tid], &a, &b, &k, &u)) user = ((unsigned long long)u.dwHighDateTime << 32 | u.dwLowDateTime) / 10000;
        std::fprintf(out, "THREAD %lu samples %u user_ms %llu\n", (unsigned long)tid, total[tid], user);
        for (auto& [rip, n] : m) std::fprintf(out, "R %lu %llx %u\n", (unsigned long)tid, rip, n);
    }
    std::fclose(out); return 0;
}

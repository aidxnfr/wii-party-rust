// SPDX-License-Identifier: GPL-3.0-or-later
// G60 batch4: five DSP logic ports (GPL-2.0-or-later notices in each file) + two SDK whole files.
// Two exact C++ lifetime/Event residues and native standard-library dependencies remain explicit.
#[path="native_g60_batch4_files/accelerator.rs"]mod accelerator;
#[path="native_g60_batch4_files/tables.rs"]mod tables;
#[path="native_g60_batch4_files/hwinterface.rs"]mod hwinterface;
#[path="native_g60_batch4_files/core.rs"]mod core;
#[path="native_g60_batch4_files/interpreter.rs"]mod interpreter;
#[path="native_g60_batch4_files/ui_screenshot.rs"]mod ui_screenshot;

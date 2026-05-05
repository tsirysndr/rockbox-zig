/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Stub CPU-info driver for the headless host build. The firmware uses
 * these only for diagnostics; returning safe dummy values is harmless.
 */

#include <stdbool.h>
#include <stddef.h>

struct cpuusage;
struct time_state;
struct cpufreq_governor;

bool current_scaling_governor(int cpu, char *g, int gsize)
{ (void)cpu; if (g && gsize > 0) g[0] = 0; return false; }

int  cpuusage_linux(struct cpuusage *u)        { (void)u; return 0; }
int  cpucount_linux(void)                      { return 1; }
int  frequency_linux(int cpu)                  { (void)cpu; return 0; }
int  min_scaling_frequency(int cpu)            { (void)cpu; return 0; }
int  current_scaling_frequency(int cpu)        { (void)cpu; return 0; }
int  max_scaling_frequency(int cpu)            { (void)cpu; return 0; }
int  cpustatetimes_linux(int cpu, struct time_state *d, int max)
{ (void)cpu; (void)d; (void)max; return 0; }

void cpufreq_available_governors(char *g, int gsize, int cpu)
{ (void)g; (void)gsize; (void)cpu; if (g && gsize > 0) g[0] = 0; }

void cpufreq_set_governor(const char *g, int cpu) { (void)g; (void)cpu; }

/* Battery stubs — real desktop battery info not needed by the engine.
 * powermgmt.c provides stubs for measures it does not track (returning -1).
 * We only need to supply the stub when powermgmt.c expects a HW implementation,
 * i.e. when CONFIG_BATTERY_MEASURE says the capability IS present. */
#include "config.h"

#if (CONFIG_BATTERY_MEASURE & PERCENTAGE_MEASURE)
int      _battery_level(void)   { return 100; }
#endif
#if (CONFIG_BATTERY_MEASURE & VOLTAGE_MEASURE)
unsigned _battery_voltage(void) { return 0; }
#endif
#if (CONFIG_BATTERY_MEASURE & TIME_MEASURE)
int      _battery_time(void)    { return 0; }
#endif

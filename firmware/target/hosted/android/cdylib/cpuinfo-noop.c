/* SPDX-License-Identifier: GPL-2.0-or-later
 *
 * Stubs for cpuinfo-linux.c / cpufreq-linux.c which we gate out of the
 * cdylib build (bionic doesn't ship cpu-features.h in default includes,
 * and we don't need cpu-info introspection from the engine — JS already
 * has Android's own /proc/cpuinfo if it ever needs it).
 *
 * Returning 1 cpu / no-frequencies is harmless: the upper layer uses
 * these only for diagnostics, not for scheduling decisions.
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

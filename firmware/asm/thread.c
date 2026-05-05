  /* First some generic implementations */
#if defined(HAVE_WIN32_FIBER_THREADS)
  #include "thread-win32.c"
#elif defined(HAVE_POSIX_THREADS)
  /* thread-posix.c is compiled as a separate TU (listed in SOURCES);
   * nothing extra to include here. */
#elif defined(HAVE_SIGALTSTACK_THREADS)
  #include "thread-unix.c"

  /* Now the CPU-specific implementations */
#elif defined(CPU_ARM_CLASSIC) || defined(CPU_ARM_APPLICATION)
  #include "arm/thread-classic.c"
#elif defined(CPU_ARM_MICRO)
  #include "arm/thread-micro.c"
#elif defined(CPU_COLDFIRE)
  #include "m68k/thread.c"
#elif defined(CPU_MIPS)
  #include "mips/thread.c"
#else
  /* Nothing? OK, give up */
  #error Missing thread impl
#endif

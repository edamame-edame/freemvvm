#include <float.h>
#include <stdint.h>
#include <stdio.h>

_Static_assert(sizeof(uint64_t) == 8, "64-bit handle representation is required");
_Static_assert(sizeof(float) == 4, "32-bit float representation is required");
_Static_assert(sizeof(double) == 8, "64-bit double representation is required");
_Static_assert(FLT_RADIX == 2 && FLT_MANT_DIG == 24 && DBL_MANT_DIG == 53,
               "Binary32 and binary64 precision is required");

int main(void) {
  printf("bootstrap C: pointer=%zu, uint64=%zu, float=%zu, double=%zu\n",
         sizeof(void*), sizeof(uint64_t), sizeof(float), sizeof(double));
  return 0;
}

#include <stdio.h>
__attribute__((weak))
void conflict() {
    puts("b");
}
void useb(){}

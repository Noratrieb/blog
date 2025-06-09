#include <stdio.h>
void conflict();
void usec();
void useb();
int main() {
  usec();
  useb();
  conflict();
}

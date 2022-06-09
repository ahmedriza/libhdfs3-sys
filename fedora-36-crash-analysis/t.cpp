// This is a minimal bit of code to show what happens
// when an empty std::vector is dereferenced.  This will
// throw an assertion failure at runtime. 
//
// Compile with:
// g++ -D _GLIBCXX_ASSERTIONS t.cpp
//
// The `_GLIBCXX_ASSERTIONS` is turned on by default in Fedora RPM builds
//
// This can be turned off by adding the following to the RPM spec file:
//
// %undefine _auto_set_build_flags
//
// libhdfs3 code has a few places where an empty std::vector is dereferenced causing
// this to fail at runtime.  They have been corrected in some of the code in this repository
// (but not all).

#include <vector>
#include <iostream>

int main() {
  std::vector<char> buf; // (128);
  // buf.resize(0);
  std::cout << "size: " << buf.size() << std::endl;
  printf("%p\n", &buf[0]);
  return 0;
}

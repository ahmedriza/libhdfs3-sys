# libhdfs3-sys

A Rust binding to the hdfs3 library from the Apache hawq project: https://github.com/ahmedriza/hawq/tree/master/depends/libhdfs3

A copy of libhdfs3 is included here. The modifications were to the cmake build files in order to get the code to compile
with recent version of dependent libraries.

The Rust binding is inspired by https://github.com/yahoNanJing/fs-hdfs

# Requirements

The main requirements are the dependencies needed by libhdfs3.
* cmake                           http://www.cmake.org/
* boost (tested on 1.53+)         http://www.boost.org/
* google protobuf                 http://code.google.com/p/protobuf/
* libxml2                         http://www.xmlsoft.org/
* kerberos                        http://web.mit.edu/kerberos/
* libuuid                         http://sourceforge.net/projects/libuuid/
* libgsasl                        http://www.gnu.org/software/gsasl/

In addition compiling the C++ test code requires the Google Test frameworks (might remove these dependencies from 
the build here since the C++ tests are not really part of the Rust binding and will simplify the dependencies required).

* gtest 
* gmock

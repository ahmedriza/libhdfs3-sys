# libhdfs3-sys

A Rust binding to the hdfs3 library from the Apache hawq project: https://github.com/apache/hawq/tree/master/depends/libhdfs3

The advantage of using libhdfs3 is that it does not use JNI and hence there's no need to have the Hadoop jars around
or the JVM system libraries in order to use HDFS.

A copy of libhdfs3 is included here. The modifications were to the cmake build files in order to get the code to compile
with recent version of dependent libraries.

The Rust binding is inspired by https://github.com/yahoNanJing/fs-hdfs

# Requirements

The main requirements are the dependencies needed by libhdfs3.
* cmake                           http://www.cmake.org/
* google protobuf                 http://code.google.com/p/protobuf/
* libxml2                         http://www.xmlsoft.org/
* kerberos                        http://web.mit.edu/kerberos/
* libuuid                         http://sourceforge.net/projects/libuuid/
* libgsasl                        http://www.gnu.org/software/gsasl/

In addition compiling the C++ test code may require the Google Test frameworks (might remove these dependencies from 
the build here since the C++ tests are not really part of the Rust binding and will simplify the dependencies required).

* gtest 
* gmock

Note that the install section of src/CMakeLists.txt was modified, as follows:

```
INSTALL(TARGETS libhdfs3-static libhdfs3-shared
        RUNTIME DESTINATION bin
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR})
INSTALL(FILES ${HEADER} DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/hdfs)
INSTALL(FILES libhdfs3.pc DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)
```

The original source had hard-coded paths such as `lib` and `include`.  This causes problems on systems such 
as Fedora/CentOS/RHEL and others where 64-bit libraries should go in `/usr/lib64` rather than `/usr/lib`. 
Using the `GNUInstallDirs` varialbes `CMAKE_INSTALL_LIBDIR` and `CMAKE_INSTALL_INCLUDEDIR` allows the correct
location to be used or overridden by the use when invoking `cmake`. 

# Note:
The `libhdfs3.tar.gz` is a tar gzipped file of the contents of the `libhdfs3` source directory.  This can be used
for generarting RPM builds.

# rvmti - JVMTI agent in rust
Dumps JITed code metadata for perf profiler. Only basic method info and line numbers are implemented for now. Method names demangling is implemented but generics are not supported yet.

Some C wrappers and bindgen are used for low level bindings.

Usage:

    perf record -k 1 java -agentpath:./librvmti.so -XX:+PreserveFramePointer -XX:+UnlockDiagnosticVMOptions -XX:+DebugNonSafepoints ...
    perf inject -i perf.data -j -o perf.data.jitted
    perf report -i perf.data.jitted

Oracle and Java are registered trademarks of Oracle and/or its affiliates. Other names may be trademarks of their respective owners.

Rust and Cargo are trademarks of the Mozilla Foundation.
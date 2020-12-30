# Getting Started with Teaclave SGX SDK
## Environment
The environment setup is documented well at the [Teaclave SGX SDK repository.](https://github.com/apache/incubator-teaclave-sgx-sdk).
Getting the environment setup is key. Make sure that you can run some of the samples in the base sgx environment inside of the docker container before proceeding.

## Project Setup
There is no clean way that I found to setup a project from scratch. I just ended up copying the [hello-rust](https://github.com/apache/incubator-teaclave-sgx-sdk/tree/v1.1.3/samplecode/hello-rust)
sample from the Teaclave SGX SDK repository and changing the names in project files. In the future there might be an easier way or tooling to make this easier, so have a look around.

## Useful Documentation
### [Intel SGX Developer Reference](https://download.01.org/intel-sgx/sgx-linux/2.12/docs/Intel_SGX_Developer_Reference_Linux_2.12_Open_Source.pdf)
This document is a great place to start. Even if you are using the rust library for sgx, the base concepts and interactions are the same.
I reccomend reading the section on the Enclave Definition Language before getting started, especially the part about how the memory gets allocated, and how you need to specify
it when writiing the EDL for a function. [EDL Section](https://download.01.org/intel-sgx/sgx-linux/2.12/docs/Intel_SGX_Developer_Reference_Linux_2.12_Open_Source.pdf#%5B%7B%22num%22%3A104%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C94.5%2C380.25%2C0%5D)

### Teaclave library documentation
For library documentation the easiest is to do `cargo doc` inside of your project directory, and to use the cargo-generated documentation. When looking at a struct or function, pay attention to the "Requirements" section.
This section will list the headers (edl and c-style) as well as the libraries required. Note that the required libraries need to be setup as part of the linking step in the makefile,
and the headers will need to be imported (or included) in your enclave's EDL file. There is also documentation on linking different libraries (and linking in general) in the aforementioned Intel SGX Developer Reference.

## Makefiles
It is important that you understand what the makefiles does at each step. The Intel SGX Developer Reference linked earlier should have an explanation for all of the sgx tools (like the signing and edger8r tools).

As your codebase grows you will (probably) start to use a wider variety of libraries inside of the enclave code. This will require you to modify the linking step of the makefile. To know what you need to link refer
to the Teaclave library documentation and the Intel SGX Developer reference
[This issue goes into a spesific case](https://github.com/apache/incubator-teaclave-sgx-sdk/issues/305).





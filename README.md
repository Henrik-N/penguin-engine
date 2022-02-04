# Penguin Engine
Vulkan Engine written in Rust.

Penguin Engine is a project I started working on in order to learn Rust and rendering. 
Since the beginning, the project has been reiterated over and restructured countless times, since I at the start didn't really know Rust nor the Vulkan API.

Currently, this is a project I am working on in tandem with [penguinengine-cpp](https://github.com/Henrik-N/penguinengine-cpp).
In this project, I've explored a bunch of different patterns and possibilities with Rust, 
but due to the fact that I didn't know the language nor the API once I started working on it, the core structure is still less than ideal. I've also explored a
lot of new things, like if an ECS-library would fit into a renderer, and because of this the project has grown quite coupled to 3rd-party libraries. Since C++20 is now
feature complete with MSVC, I've decided to use some of the concepts I've learned in this project on a clean slate using C++20. Doing that is forcing me not to simply
reuse any code from this project. Several of the new features in C++20 are also very similar to features in Rust, and I think learning how to take advantage of them
is going to benefit my knowledge in both languages. 
As I rebuild a better ground structure in the C++ project, and one with less dependencies, I'll push those changes into this repo as well. 

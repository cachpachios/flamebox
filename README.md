# DEPRECATED
This was a cool project to get started with MicroVMs, however the underlying idea to extract the container INTO a the rootfs of the VM has drawbacks.
You would need a fully OCI compliant init process inside the VM anyways, and at that point why not just use runc inside the VM, similar to how Kata does things.
Also out of lazyness, the host kernel was utilized to create mount points to the filesystem when extracting the container image. That should probably happen fully in user-space... Yeah never production ready stuff here.

# FlameBox

This is an POC of creating the absolute minimal container "runtime" for OCI container images (i.e Docker) in MicroVms powered by [Firecracker](https://github.com/firecracker-microvm/firecracker) using Rust.

Obviously this is a personal POC project, but public for anyone who find it interesting, educational or useful.

This is not a novel idea by any stretch of the imagination, for real implementations see Kata Containers, Flintlock,
Firecracker Containerd, etc.

## Features

- [x] Pull OCI images from registries (Docker Hub) (just nice to have...)
- [x] Extract image layers into a ext4 filesystem
- [ ] Provide suitable /sbin/init and host etc
- [x] Boot the VM with the filesystem
- [ ] Networking...

Maybe future extensions would be to use something like OverlayFS to allow ephemeral changes to the container filesystem, as expected with Docker. Another to create a virtual network stack and attach the VM to it.

## Getting started

### Build requirements

Obv, install cargo & rust. Also (ubuntu) install `libsquashfs-dev`.


## Why?

### Regular Containers

A "regular" container engine (such as Docker) works something like this:

- We extract the image layers into a OverlayFS filesystem which also is writable.
- We start the container entrypoint and use `chroot` to change the root to the filesystem.
- Namespaces and cgroups are used to isolate the container from the host and manage resources.

This means that it still runs in the same kernel as the host and we are just relying on kernel features for isolation. This is generally good enough, but breakouts have been found in the past and its not that more will be found in the future. So for multi-tenant environments, that's not ideal.

Using a proper VM for isolation, with its own kernel, would be more secure.

### MicroVMs

MicroVMs is basically just like any other VM but more lightweight and is therefore faster, especially to boot up.
This is done by limiting the number of devices to absolute bare minimum with only highly performant virtualized network and storage devices, avoiding emulation as far as possible, and also not going through the whole regular BIOS boot process.

But containers are awesome, its a scalable, reproducible and easy way to package a application and all its dependencies (all the way to what we "traditionally" think of as the OS) into a single image.

So why not combine the two to run containers in an isolated manner?

## How?

In the end, extracting the container image into its own filesystem and running a kernel on top of that will give you the same environment as in the container, but with the added security of not sharing the kernel with the host.

So by doing exactly that, we can run a container in a MicroVM.

Step by step:

- The image layers is extracted into a SquashFS filesystem.
- A VM is started with the SquashFS as the root filesystem, marked read-only.

Success! Simple, right?

Extending it to allow tun-tap networking, maybe a writable layer with OverlayFS on the root fs and additional mounts would be a bit more work, but thats also pretty straight forward.

Of course a proper container runtime should follow the OCI runtime spec which complicates things significantly.
But thats not the goal here.

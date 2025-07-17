Name:           rockbox
Version:        2025.07.17
Release:        1%{?dist}
Summary:        High quality audio player

License:        GPL-2.0

BuildArch:      x86_64

Requires: SDL2, freetype, libunwind, alsa-utils, alsa-lib-devel

%description
Rockbox open source high quality audio player

%prep
# Prepare the build environment

%build
# Build steps (if any)

%install
mkdir -p %{buildroot}/usr/local/bin
mkdir -p %{buildroot}/usr/local/lib
mkdir -p %{buildroot}/usr/local/share
cp -r %{_sourcedir}/amd64/usr %{buildroot}/

%files
/usr/local/bin/rockbox
/usr/local/bin/rockboxd
/usr/local/lib/rockbox/*
/usr/local/share/rockbox/*


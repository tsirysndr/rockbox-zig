FROM fedora:latest

RUN dnf install -y meson gcc g++ python3-pip gobject-introspection-devel \
	python3-devel python3-gobject git diffutils xorg-x11-server-Xvfb \
	appstream-devel dbus-x11 "dnf-command(builddep)" glslc
RUN dnf build-dep -y gtk4 libadwaita
RUN pip3 install furo mypy sphinx coverage black isort

COPY install_deps.sh .
RUN ./install_deps.sh

# The version on PyPI is very old and doesn't install. Use the upstream package registry instead.
RUN pip install pythonfuzz --extra-index-url https://gitlab.com/api/v4/projects/19904939/packages/pypi/simple

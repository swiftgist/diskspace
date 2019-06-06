%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: diskspace
Summary: Friendly command line utility for finding the largest files and directories
Version: @@VERSION@@
Release: 1
License: GPLv3
Group: Applications/System
Source0: %{name}-%{version}.tgz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -c

%build
/bin/tar xvzf %{SOURCE0}

%install
install -d -m 755 %{buildroot}%{_bindir}
install -m 555 target/release/ds %{buildroot}%{_bindir}
install -d -m 755 %{buildroot}%{_docdir}/%{name}
install -m 644 LICENSE %{buildroot}%{_docdir}/%{name}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
%{_docdir}/%{name}/LICENSE

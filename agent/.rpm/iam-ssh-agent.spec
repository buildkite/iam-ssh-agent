%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: iam-ssh-agent
Summary: ssh-agent compatible daemon that forwards list-keys and sign-data operations to an API Gateway backend, access controlled by the caller's IAM identity.
Version: @@VERSION@@
Release: @@RELEASE@@
License: BSD
Group: Applications/System
Source0: %{name}-%{version}.tar.gz

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*

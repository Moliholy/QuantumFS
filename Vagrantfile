# -*- mode: ruby -*-
# vi: set ft=ruby :

VAGRANT_COMMAND = ARGV[0]

# All Vagrant configuration is done below. The "2" in Vagrant.configure
# configures the configuration version (we support older styles for
# backwards compatibility). Please don't change it unless you know what
# you're doing.
Vagrant.configure(2) do |config|
  config.vm.provider "virtualbox" do |v|
    host = RbConfig::CONFIG['host_os']

    if host =~ /darwin/
      cpus = [`sysctl -n hw.ncpu`.to_i / 2, 1].max
      # sysctl returns Bytes and we need to convert to MB
      mem = `sysctl -n hw.memsize`.to_i / 1024 / 1024 / 4
    elsif host =~ /linux/
      cpus = [`nproc`.to_i / 2, 1].max
      # meminfo shows KB and we need to convert to MB
      mem = `grep 'MemTotal' /proc/meminfo | sed -e 's/MemTotal://' -e 's/ kB//'`.to_i / 1024 / 4
    else # windows
      cpus = 2
      mem = 1024
    end

    v.customize ["modifyvm", :id, "--memory", mem]
    v.customize ["modifyvm", :id, "--cpus", cpus]
  end

  if VAGRANT_COMMAND == "ssh"
    config.ssh.username = 'vagrant'
  end

  config.vm.define "ubuntu" do |ub|
    ub.vm.box = "geerlingguy/ubuntu1804"
    ub.vm.network "private_network", ip: "192.168.33.15"
    ub.vm.synced_folder '.', '/vagrant'
    ub.vm.provision "shell", path: ".provision_ubuntu.sh"
  end
end

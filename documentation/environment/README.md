# Provision of a `platys`-ready environment 

A `platys`-ready environment can be provisioned to various environments. Find the options and description here:

  * [AWS Lightsail](./Lightsail.md) - AWS Lightsail is a service in Amazon Web Services (AWS) with which we can easily startup an environment and provide all the necessary bootstrapping as a script. Monthly price of USD 80.- (16GB RAM) or USD 160.- (32 GB RAM).
  * [Azure VM (t.b.d.)]() - Run it inside an Azure VM
  * [Local Docker](./LocalDocker.md) - you have a local Docker and Docker Compose setup in place, either on Windows, on Mac or Linux, which you want to use
  * [Local Virtual Machine with Docker](./LocalVirtualMachine) - a Virtual Machine with Docker and Docker Compose pre-installed will be distributed at by the course infrastructure. You will need 50 GB free disk space.

## Post Provisioning

These steps are necessary after the starting the docker environment. 

### Add entry to local `/etc/hosts` File

To simplify working with the Streaming Platform and for the links below to work, add the following entry to your local `/etc/hosts` file. 

```
40.91.195.92	dataplatform
```

Replace the IP address by the public IP address of the Docker host. 

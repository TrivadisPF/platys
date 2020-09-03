![](tri_logo_high.jpg)

[![License](http://img.shields.io/:license-Apache%202-blue.svg)](http://www.apache.org/licenses/LICENSE-2.0.txt)
[![Code Climate](https://codeclimate.com/github/codeclimate/codeclimate/badges/gpa.svg)](https://codeclimate.com/github/TrivadisPF/modern-data-platform-stack)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](http://makeapullrequest.com)
[![](https://img.shields.io/static/v1?logo=slack&logoColor=959DA5&label=Slack&labelColor=333a41&message=join%20conversation&color=3AC358)](https://join.slack.com/t/platys/shared_invite/zt-gc31af0n-cLHnzSaBSqS~IsIMSK6SKg)

# platys - Trivadis Platform in a Box
Copyright (c) 2018-2020 Trivadis

## Why `platys`?

There is no question that running workloads in containers simplifies deployment of applications. But one container is never enough, you need more than one container to implement a working solution, such as database, business logic, event broker ... and you need a Container Orchestration to manage these containers.

Today Kubernetes is the the most popular container orchestrator, but it comes with a lot of complexity. For production setups, Kubernetes is definitely one way to go. But for local, development or small-scale Proof-of-Concepts, we like to use Docker Compose, a very simple approach to container orchestration. With Compose, you use a YAML file to configure your application’s services.

Especially as a consultant, coach, trainer, technology evangelist, you will be using different Compose setups for different environments.

So the longer you use Docker Compose, the more of these YAML files you get and to maintain them is quite a challenge: 
 
 * But how do you easily upgrade to a new version of a container, i.e. Apache Kafka?
 * Do you manually have to go through all of these Compose files, which is a lot of work and prone to errors? 
 * What if you want to add a new service to one environment and you know that you have used it previously in another environment?
 * Do you copy-paste configs from one YAML to another?
 * How do you make sure that one service from another environment will work with all the configs and port settings of your other environment?

For these and some other challenges we were looking for a better solution: 
 
 * Wouldn't it be easier to generate the Docker Compose YAML, based on a simple configuration with some ON/OFF switches of all supported services? 

Enter the world of `platys`...

## What is `platys`?

`platys` is a tool for generating and provisioning Modern Data Platforms based on [Docker](https://www.docker.com/get-started) and [Docker Compose](https://docs.docker.com/compose/). 

Its main use is for small-scale Data Lab projects, Proof-of-Concepts (PoC) or Proof-of-value (PoV) projects as well as trainings.

The user of `platys` can choose which services to use from a list of supported services and generate a fully working `docker-compose.yml` file including all necessary configuration files.

This replaces our old approach, where we only had a static `docker-compose.yml` file with all services enabled by default. By generating the `docker-compose.yml`, the user has very fine-grained control on which services to include for a given platform. 
 
## How does `platys` work?

The following diagram shows the building blocks of `platys` and the basic flow when working with it. 

A concrete _Platform_ is always generated based on a given _Platform Stack_. A platform stack defines the set of available and usable services and has a name and a version. 

![platys](./documentation/images/platys-tool.png)

1. Initialize a new _Platform_ context by specifying a _Platform Stack_. Optionally a set of services to be enabled can be specified with the `init` command. 
2. Optionally edit the `config.yml` to enable services and change default values of configuration settings.
3. Generate the artefacts for the platform (mainly the `docker-compose.yml` but also some configuration files) by running the `gen` command.
4. Run `docker-compose up` to start your platform.


Currently the following Platform Stack is available:

* [`modern-data-platform`](https://github.com/TrivadisPF/platys-modern-data-platform) - a Platform Stack for supporting a Modern (Analytical) Data Platforms

In the future, other platform stacks might be added.

## Where can I run `platys`?

The **platys** toolset runs on Windows, macOS and 64-bit Linux. 

`platys` runs the generator (supporting given Platform Stack) as a Docker container in the background. Therefore you need to have [Docker](https://www.docker.com/get-started) installed on the machine where you create the Platform. To run the Platform, you also need to have [Docker Compose](https://docs.docker.com/compose/) installed on the target machine, which can be different to the one where you generate the platform.  

See [Installing platys](./documentation/install.md) for how to install `platys` and then the [Getting Started with Platys and the Modern Data Platform Stack](https://github.com/TrivadisPF/platys-modern-data-platform/blob/master/documentation/getting-started.md) for how to use `platys`.

## Where can I run a generated platform ?

The generated platform can be provisioned either locally or in the cloud. See [Provisioning of Modern Data Platform](./documentation/environment/README.md) for various versions of how to deploy the platform. 

## Changes 
See [What's new?](./documentation/changes.md) for a detailed list of changes.

## Documentation

**Usage**

* [Installing platys](./documentation/install.md)
* [Getting Started with `platys` and the `modern-data-platform` platform stack](https://github.com/TrivadisPF/platys-modern-data-platform/blob/master/documentation/getting-started.md)
* [Frequently Asked Questions](./documentation/faq.md)
* [Command line reference](./documentation/command-line-ref.md)
* [Glossary of Terms](./documentation/glossary.md)

**Development**

* [Service Design Decisions](./documentation/service-design.md)
* [Creating and maintaining a Platform Stack](./documentation/creating-and-maintaining-platform-stack.md)





# Frequently Asked Questions

If you don’t see your question here, feel free to add an issue on GitHub. 

## How can I add additional services, not supported by a Platform Stack?

If you have a service which is not supported by the Platform Stack you base on, then you can add it by using [the method of using multiple compose files](https://docs.docker.com/compose/extends/#multiple-compose-files) and creating a `docker-compose.override.yml` file.

Just create a new file in the same folder as the `docker-compose.yml`, name it `docker-compose.override.yml` and add the a header similar to the one, where the version should be the same as the one as in the generated `docker-compose.yml` file (currently `3.0` by default).

```
version: "3.0"

services:
  <add your service definitions here ...>
```

This is much better than manually changing/adapting the generated `docker-compose.yml`, which you should avoid by all means. 

Of course you can also ask for inclusion of the service by creating a new issue on this GitHub project. 


   
## `platys` documentation

* [Getting Started with `platys` and the `modern-data-platform` platform stack](../platform-stacks/modern-data-platform/documentation/getting-started.md)
* [Explore the full list of Platys commands](overview-platys-command.md)

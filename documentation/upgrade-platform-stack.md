# Upgrade to a new platform stack version

If you have been using a given platform stack for some time, at some point you might want to upgrade to a new platform stack version. 

Of course you can always completely generate a new stack and then enable all the services of the previous version. If on the other hand you prefer to upgrade an existing stack to a later version, then you find the following helpful. 

Let's say you have a stack with the following header in the `config.yml`:

```
      # Default values for the generator
      # this file can be used as a template for a custom configuration
      # or to know about the different variables available for the generator
      platys:
          platform-name: 'demo'
          platform-stack: 'trivadis/platys-modern-data-platform'
          platform-stack-version: '1.8.0'
          structure: 'flat'
```

To upgrade to version `1.9.0`, you can just change the value of the `platform-stack-version` property and generate the platform again. 

```
platys gen
```

You will see that the docker image of the new generator version will be downloaded and the platform stack will be generated based on that. 

You might also want to check the configuration document for configuration properties which have been added with the new version and add them to the `config.yml` file. 
   
## `platys` documentation

* [Explore the full list of Platys commands](overview-platys-command.md)

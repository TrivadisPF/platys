# Service Design Decisions

This document describes some design decisions in the way the services are built by the generator.

## Service Variants

If you have a service, which can be deployed in various ways and for each one there is a separate docker image, then we use the concept of service editions. 

When working with service editions, the `<service>_enabled` flag generically enables the service (where `<service>` is replaced by the name of the service) independent of the edition in use. Then there is a second text property, specifying the edition.

```
<service>_enabled=[true | false]
<service>_edition=['edition-1' | 'edition-2' | ... ] 
```
 
An example where this concpet is applied can be found with the `Jupyter` service:

```
JUPYTER_enable: false
# one of 'minimal', 'r', 'scipy', 'tensorflow', 'datascience', 'all_spark'
JUPYTER_edition: 'minimal'
``` 

## Multiple Service Major Versions

Normally, a service version is just a property which can be changed from one version to another and represents the tag of the docker image which should be used. This is the case with minor version changes. 

In case where a service has a major new version, then we can also duplicate the service completely. We might choose this route if the new service is no longer backward compatible with the previous one or if the new service is released as a preview or beta version and it should have its own default versions. 

In that case, we have separate enable flags, one for each major version. 

```
<service>_enabled=[true | false]
<service>2_enable=[true | false]
```

An example where this concept is applied can be found with the `hivemq` service, where we allow both the HiveMQ v3 and v4 to be enabled, without having to change the version property:

```
HIVEMQ3_enable: false #needs MQTT_ENABLE = true
HIVEMQ4_enable: false #needs MQTT_ENABLE = true
```

## Omit a value, if not defined

Let's say `s3endpoint` is a variable, which can be undefined. In that case you can use the omit clause to to generate the given line at all

```
environment:
  ENDPOINT: '{{s3Endpoint | default(omit) }}'
```

Let's say `s3endpoint` is a variable, which can be undefined. If it is defined, we need a value of `true` and otherwise we want to omit generating the given line. 

```
environment:
  USE_S3: {{'true' if s3Endpoint is defined else omit }}


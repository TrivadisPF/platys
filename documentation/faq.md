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

## How can I create a new stack automatically in a scripted way?

Instead of initialising a platys stack, manually editing the `config.yml` and then generating the stack, you can do it automatically, assuming that you have the configuration ready:

```bash
mkdir -p platys-demo
cd platys-demo

cat > config.yml << EOL
      platys:
        platform-name: 'platys-platform'
        platform-stack: 'trivadis/platys-modern-data-platform'
        platform-stack-version: '1.17.0-preview'
        structure: 'flat'

      # ========================================================================
      # Global configuration, valid for all or a group of services
      # ========================================================================
      # Timezone, use a Linux string such as Europe/Zurich or America/New_York
      use_timezone: ''
      # Name of the repository to use for private images, which are not on docker hub (currently only Oracle images)
      private_docker_repository_name: 'trivadis'
      # UID to use when using the "user" property in a service to override the user inside the container
      uid: '1000'

      # Optional environment identifier of this platys instance, by default take it from environment variable but can be changed to hardcoded value. 
      # Allowed values (taken from DataHub): dev, test, qa, uat, ei, pre, non_prod, prod, corp
      env: '${PLATYS_ENV}'

      data_centers: 'dc1,dc2'
      data_center_to_use: 0

      copy_cookbook_data_folder: true
      
      # ========================================================================
      # Platys Services
      # ========================================================================
      #
      # ===== Trino ========
      #
      TRINO_enable: false
      # "single" or "cluster" install
      TRINO_install: single
      TRINO_workers: 3
      # either starburstdata or oss
      TRINO_edition: 'starburstdata'
      TRINO_auth_enabled: false
      TRINO_auth_use_custom_password_file: false
      TRINO_auth_use_custom_certs: false
      TRINO_auth_with_groups: false
      TRINO_access_control_enabled: false
      TRINO_hive_storage_format: ORC
      TRINO_hive_compression_codec: GZIP
      TRINO_hive_views_enabled: false
      TRINO_hive_run_as_invoker: false
      TRINO_hive_legacy_translation: false
      TRINO_kafka_table_names: ''
      TRINO_kafka_default_schema: ''
      TRINO_event_listener: ''
      TRINO_postgresql_database: ''
      TRINO_postgresql_user: ''
      TRINO_postgresql_password: ''
      TRINO_oracle_user: ''
      TRINO_oracle_password: ''
      TRINO_sqlserver_database: ''
      TRINO_sqlserver_user: ''
      TRINO_sqlserver_password: ''
      TRINO_with_tpch_catalog: false
      TRINO_with_tpcds_catalog: false
      TRINO_with_memory_catalog: false
      TRINO_starburstdata_use_license: false
      TRINO_starburstdata_enable_data_product: false
      TRINO_additional_catalogs: ''
      TRINO_additional_connectors: ''

      # Trino-CLI is enabled by default
      TRINO_CLI_enable: true
EOL
      
platys gen

export DOCKER_HOST_IP=nnn.nnn.nnn.nnn
export PUBLICH_IP=nnn.nnn.nnn.nnn

docker-compose up -d
```
   
## `platys` documentation

* [Getting Started with `platys` and the `modern-data-platform` platform stack](../platform-stacks/modern-data-platform/documentation/getting-started.md)
* [Explore the full list of Platys commands](overview-platys-command.md)

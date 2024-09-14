# Base Pipeline

[![fluentci pipeline](https://img.shields.io/badge/dynamic/json?label=pkg.fluentci.io&labelColor=%23000&color=%23460cf1&url=https%3A%2F%2Fapi.fluentci.io%2Fv1%2Fpipeline%2Fbase_pipeline&query=%24.version)](https://pkg.fluentci.io/base_pipeline)
![deno compatibility](https://shield.deno.dev/deno/^1.42)

This repository contains a minimal pipeline for a [Fluent CI](https://fluentci.io) project. It is intended to be used as a template for new projects.
Reusing this template will allow you to get started with Fluent CI in a matter of minutes, just run the following command:

```bash
fluentci init
```

## Files Tree Layout

```plaintext
src
|-- helpers.ts  : Contains helper functions
|-- jobs.ts     : Contains the job definitions
|-- mod.ts      : This is the entry point of the module
|-- pipeline.ts : Contains the pipeline definition
`-- runner.ts   : Contains the runner function
```

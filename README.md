# Machine API

This API intends to provide a standardized local interface to any machines used for manufacturing physical things.

## Config file

Here is a sample config file:

```toml
[bambulabs]
machines = [
    { id = "YOUR_ID_HERE", access_code = "YOUR_ACCESS_CODE_HERE", slicer_config = "./config/bambu/" },
    { id = "YOUR_ID_HERE", access_code = "YOUR_ACCESS_CODE_HERE", slicer_config = "./config/bambu/" },
]

[formlabs]
```

The cli looks by default for a file called `machine-api.toml` in the current
directory. You can also specify a different file with the `--config` flag.


## Running machine API

You can run this server locally assuming rust is installed:

```
cargo run server --address 0.0.0.0:8585
```

The full API is described by the OpenAPI spec, but to start you can list the connected machines:

```bash
$ curl http://localhost:8585/machines
# [{"port":"/dev/ttyACM0","id":"CZPX2418X004XK68718","manufacturer":"Prusa Research (prusa3d.com)","model":"Original Prusa i3 MK3"}]
```

The ID is what you'll use to identify the machine. You can use that ID to start a print job. 

For example, providing both an STL as `file`, and `params` as a json object with `machine_id` the same as above:

```bash
curl -X POST -F file=@input.stl -F 'params={"machine_id": "CZPX2418X004XK68718", "job_name": "my-cool-job"}' http://localhost:8585/print
```

Note: you may need to allow user permissions to USB devices. Alternatively, you can just run the server as root.

## machine-api CLI

You can also use machine-api as a CLI. `cargo run` with no parameters will give the available options.

## Regenerating the OpenAPI definition file

```bash
EXPECTORATE=overwrite cargo test --all openapi
```

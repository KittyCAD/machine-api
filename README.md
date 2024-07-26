# Machine API

This API intends to provide a standardized local interface to any machines used for manufacturing physical things.


## Running machine API

You can run this server locally assuming rust is installed:

```
cargo run server --address 0.0.0.0:8585
```

The full API is described by the OpenAPI spec, but to start you can list the connected printers:

```
$ curl http://localhost:8585/printers
[{"port":"/dev/ttyACM0","id":"CZPX2418X004XK68718","manufacturer":"Prusa Research (prusa3d.com)","model":"Original Prusa i3 MK3"}]
```

The ID is what you'll use to identify the printer. You can use that ID to start a print job. 

For example, providing both an STL as `file`, and `params` as a json object with `printer_id` the same as above:

```
curl -X POST -F file=@input.stl -F 'params={"printer_id": "CZPX2418X004XK68718"}' http://localhost:8585/print
```

Note: you may need to allow user permissions to USB devices. Alternatively, you can just run the server as root.

## machine-api CLI

You can also use machine-api as a CLI. `cargo run` with no parameters will give the available options.

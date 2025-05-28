# Event Tracker

A minimal, thread-safe event tracking service.  Supports event submission and querying via a simple HTTP API.

## Tech Stack
- Built in **Rust**
    - **Actix_web** for HTTP service
        - **Actix_rt** for async runtime
    - **thiserror** for unified error handling
    - **RwLock** for thread safety
    - **Serde** for Type serial and deserialization
    - **Actix_governor** for rate limiting
- **Docker** for deployment

## Project Structure
```text
src/
 - api.rs -> HTTP route definition
 - error.rs -> Application error types
 - main.rs -> Entry point
 - lib.rs -> Re-exports for integration tests
 - model.rs -> Data models (Event, EventQuery)
 - storage.rs -> Storage trait + in-memory implementation
tests/
 - api_get_requests.rs -> integration tests for GET requests
 - api_post_requests.rs -> integration tests for POST requests
 - rate_limiting.rs -> simple test of the rate limiting middleware
 ```

## Data Storage

Current 1.0 release uses an in-memory storage mechanism that stores data in a `Hashmap<UUID,Event>` and is wrapped in an Arc to be used in the webserver.

A public trait was created so that swapping the in-memory data store with something with persistence (e.g. Sqlite or Postgres), so that impact is minimally felt across the rest of the app.  A new implementation should be easily swappable.

_Note: a thread pool or connection pool should be considered for persistent backends._

## API

Webserver exposes 3 services at one endpoint:
- '**POST** /events' - Creates a new event using the following payload: {"event_type: "[string]"", "timestamp":"[valid UTC datetime string]", "payload":"[json object]"}.  A UUID is added once inserted for faster querying.  Returns a new event object.
- '**GET** /events' - Returns a list of all events currently stored.  Accepts query parameters to filter the results.  Current query parameters are: 'event_type', 'start' (time), and 'end' (time). _Ex:`"/events?start=2025-01-02T00:00:00Z&end=2025-01-02T23:59:59Z&event_type=login"`_
- '**GET** /events/{id}' - Returns the event for the given UUID.

## Design Notes

- The `EventStore` trait abstracts storage to support future persistence layers (e.g. SQLite, Postgres).
- `RwLock` ensures concurrent, thread-safe access to the in-memory store with minimal overhead.
- Errors are centralized via `thiserror` for consistency across API and internal logic.
- UUIDs allow efficient querying and decouple internal identity from payload contents.
- Integration tests validate API behavior and data filtering logic across edge cases.
- For rate limiting, during development, `actix_governor` was chosen due to its simplicity and support for in-memory usage. For production, a more scalable approach would be considered for distributed rate limiting and persistence, such as `actix_limitation` with a Redis backend.


## Running the application
### Local
Use `cargo run`

The application will start on `localhost:8080` or `127.0.0.1:8080`.

Once running, use `curl` in a terminal to execute requests:
```Bash
curl -X POST http://127.0.0.1:8080/events \
  -H "Content-Type: application/json" \
  -d '{
    "event_type": "login",
    "timestamp": "2025-01-01T12:00:00Z",
    "payload": {
      "user_id": 1
    }
  }'
```
Can query:
```Bash
curl -X GET http://127.0.0.1:8080/events?event_type=login&start=2025-01-01T12:00:00Z&end=2025-01-01T23:59:59Z
```

Use event ID returned from POST:
```Bash
curl -X GET http://127.0.0.1:8080/events/0d67f74f-1090-4425-89f1-9196be25d24b
```

### Docker

From the project root directory, build the docker image:
```bash
docker build -t event-tracker:latest .
```

Once built, run it with the following command:
```bash
docker run -e BIND_ADDRESS=0.0.0.0:8080 -p 8080:8080 event-tracker
```

The App requires an environment variable so it can correctly bind to any interface when run inside a container. 

Reaching the webserver uses the same curl commands listed when running locally.

## Testing

Run all unit and integration tests with:

```bash
cargo test
```
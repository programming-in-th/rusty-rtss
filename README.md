# Rusty RTSS
Realtime message delivery with rust.

# Installation
1. Setup env in `.cargo/config.toml`
2. Run with cargo
```sh
cargo run
```

Note that running with other method requires user to manage environment variable appropriately

## Postgres setup
1. Create notify function
example
```sql
CREATE OR REPLACE FUNCTION public.notify()
  RETURNS trigger
  LANGUAGE plpgsql
AS $function$
	BEGIN
  		PERFORM pg_notify('listen_channel', CAST(NEW.id as TEXT));
  	return new;
	END;
$function$
;
```

2. Create notify trigger
```sql
CREATE OR REPLACE TRIGGER on_insert AFTER
INSERT
    ON
    public.table FOR EACH ROW EXECUTE FUNCTION notify()
;
```
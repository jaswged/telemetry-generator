# Telemetry Generator

## About

Generate sample rocket telemetry data to play around with.

## To Build

`cargo build`

## To Run

```bash
cargo run -- generate --output output  --khz 1
```

### Query the Parquet

```bash
# Count
duckdb -c "select count(*) from read_parquet('output.parquet')"

# Tail of file
duckdb -c "select * from read_parquet('output.parquet') order by timestamp desc limit 10"

# Distinct sensor types
duckdb -c "select distinct(sensor_type) from read_parquet('one_hertz.parquet')"
```

## ToDos

- [ ] Remove/reduce sensor name from exported data. Would get mapped by application. This would reduce storage space. Document how much
- [ ] Create Jupyter notebook to graph out the squiggles and see the data
- [ ] Influx db insertions
- [ ] Add unit tests
- [ ] Add clippy check to github pipeline. (See blue example)
- [ ] ...

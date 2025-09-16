# Telemetry Generator

## About

Generate sample rocket telemetry data to play around with.

## To Build

`cargo build`

## To Run

Example run generation at 1,000 Hz for 60 seconds. This creates a dataset with 1,620,000 rows

```bash
cargo run --release -- generate --khz 1 -d 60
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

- [ ] Save data to file while running to avoid hitting ram limits
- [x] Remove ability to specify output file and instead construct from run parameters
- [ ] Create Jupyter notebook to graph out the squiggles and see the data
- [ ] Influx db insertions
- [ ] Add unit tests
- [ ] Add clippy check to github pipeline. (See blue example)
- [ ] ...

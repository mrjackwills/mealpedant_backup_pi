<p align="center">
	<img src='./.github/logo.svg' width='200px'/>
</p>

<p align="center">
	<h1 align="center">mealpedant backup pi</h1>
</p>

<p align="center">
	A simple backup service for mealpedant to a local device, powered by <a href='https://www.staticpi.com' target='_blank' rel='noopener noreferrer'>staticpi.com</a>
</p>

<p align="center">
	Built in <a href='https://www.rust-lang.org/' target='_blank' rel='noopener noreferrer'>Rust</a>,
	for <a href='https://docker.com' target='_blank' rel='noopener noreferrer'> Docker</a>
	<br>
	see the accompanying <a href='https://www.github.com/mrjackwills/mealpedant_backup_server' target='_blank' rel='noopener noreferrer'>server client</a>

</p>

## Required software

1) <a href='https://www.staticpi.com/' target='_blank' rel='noopener noreferrer'>staticPi</a> - simple, secure, messaging service
2) <a href='https://docker.com/' target='_blank' rel='noopener noreferrer'>Docker</a> - container runtime


| directory | reason|
| --- | --- |
|```~/mealpedant_backup/```			| Location of client|
|```~/mealpedant_backup/backups```	| Location of backups |
|```~/mealpedant_backup/logs```		| Location of logs |
|```~/mealpedant_backup/.env```		| environmental variables, make sure in production mode|


## Run step
1) ```./run.sh``` build, or re-build, docker container


## Tests

```sh
cargo test -- --test-threads=1 --nocapture


# Watch for test that start some_prefix
cargo watch -q -c -w src/ -x 'test some_prefix_ -- --test-threads=1 --nocapture'
```
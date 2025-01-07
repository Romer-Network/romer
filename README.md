# RØMER

Commit to a secret log and agree to its hash.

# Usage (Run at Least 3 to Make Progress)

## Participant 0 (Bootstrapper)

**UNIX like**
```bash
cargo run --release -- --me 0@3000 --participants 0,1,2,3 --storage-dir /tmp/log/0
```

**Windows**
```bash
cargo run --release -- --me 0@127.0.0.1:3000 --participants 0,1,2,3 --storage-dir \data\\romer_log\\0 --latitude=-28.0167 --longitude=153.4000
```

## Participant 1

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 1@3001 --participants 0,1,2,3 --storage-dir /tmp/log/1
```

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 1@127.0.0.1:3001 --participants 0,1,2,3 --storage-dir \data\\romer_log\\1 --latitude=-28.0167 --longitude=153.4000
```

# Participant 2

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 2@3002 --participants 0,1,2,3 --storage-dir /tmp/log/2
```

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 2@127.0.0.1:3002 --participants 0,1,2,3 --storage-dir \data\\romer_log\\2 --latitude=-28.0167 --longitude=153.4000
```

# Participant 3

```bash
cargo run --release -- --bootstrappers 0@127.0.0.1:3000 --me 3@3003 --participants 0,1,2,3 --storage-dir /tmp/log/3
```


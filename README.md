# LAN IM Server

![build](https://github.com/Tramis/simple-im/actions/workflows/rust.yml/badge.svg)

- codes are basically copied from [an example of Actix repositry](https://github.com/actix/examples/tree/master/websockets/chat)

just for practice

not only for LAN in theory, it's just my use scene

## Usage

```bash
.
├── chatroom
├── b
│   └── statics
│       ├── b.css
│       └── a.js
└── a
    └── index.html

# for directory tree as above
user@user:path/to/bin$ ./chatroom -d b/statics -i a/index.html
```

## Compile

need:

- -lsqlite3

## Update

- add parameter to specify index.html and static resouces (`clap`)
- add compiling-config log (`log4rs`)

## TODO

- ~~support: records~~
- support: chat rooms
- ~~highly performance with rust async runtime, such as tokio~~
- support: upload files
- support: image
- support: rich-text
- ~~support: yaml config file~~

### Small TODO

- ~~complete record~~
- rust build workflow add sqlite3
- make runable docker file

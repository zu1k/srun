# SDUSrun

[![GitHub stars](https://img.shields.io/github/stars/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/network)
[![GitHub issues](https://img.shields.io/github/issues/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/issues)
[![Release](https://img.shields.io/github/release/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/releases)
[![Build](https://github.com/zu1k/sdusrun/actions/workflows/build.yml/badge.svg)](https://github.com/zu1k/sdusrun/actions/workflows/build.yml)
[![GitHub license](https://img.shields.io/github/license/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/blob/master/LICENSE)

Srun authentication system login tools. [compatible versions](https://github.com/zu1k/sdusrun/issues/4)

## Features

- Support both command line and config file
- Multiple IP acquisition methods
  - User Specified
  - Auto detect
  - User select
  - Query by NIC name
- Support strict bind
- Support multiple users login, suitable for multi-dial
- Support multi CPU architecture
- Support multi system

## Usage

[Pre-built binaries](https://github.com/zu1k/sdusrun/releases)

### CMD mode

```
./sdusrun login -u USERNAME -p PASSWORD -i IP [-s AUTH_SERVER]
```

`AUTH_SERVER` should contain protocols, e.g. `http://10.0.0.1`.

#### Which IP to be authorized?

SDUSrun support three methods of specifying IP:

- use `-i IP` to specify ip
- use `-d` to auto detect ip
- use `--select-ip` to select ip

##### specify IP

You need to check the IP address of each network interfaces in advance and choose the correct IP to be authorized.

##### detect IP

SDUSrun support automatic IP detection, it determines the IP address from the information returned by the authentication server.

This is useful in cases where you only have one IP address to authorize.

If you are multidialing and have multiple legitimate IPs at the same time, you need to authorize multiple IPs at the same time, this method will not authorize all IPs properly.

##### select IP

This method is similar to the first method, except that it saves you the trouble of manually querying all the IPs.

SDUSrun will query all the legitimate IPs in advance and then print a list of IPs for you to choose from.

```sh
$ ./sdusrun login -u USERNAME -p PASSWORD --select-ip
Please select your IP:
    1. 192.168.226.5
    2. 10.27.196.218
    3. 172.16.150.1
    4. 192.168.128.1
    5. 198.10.0.1
2
you choose 10.27.196.218
...
```

Please note that when your computer has only one IP that can be authorized, we will simply omit the selection process and use this IP.

### Using a Config

Usually, it is sufficient to specify the information directly using command line parameters.

In order to meet the needs of multi-dial users, SDUSrun support reading multiple user information from a config file.

```
./sdusrun login -c config.json
```

config file template

```json
{
    "server": "http://10.0.0.1",
    "strict_bind": false,
    "double_stack": false,
    "retry_delay": 300,
    "retry_times": 10,
    "n": 200,
    "type": 1,
    "acid": 12,
    "os": "Windows",
    "name": "Windows 98",
    "users": [
        {
            "username": "username1",
            "password": "password1",
            "ip": "10.1.2.3"
        },
        {
            "username": "username2@cmcc",
            "password": "password2",
            "if_name": "macvlan1"
        }
    ]
}
```

As you can see, we support `ip` or `if_name`.

If your IP will not change, you can use `ip` to specify directly.

But for multi-dial, IP may be automatically assigned by DHCP and may change, at this time we suggest to use `if_name` to specify the corresponding NIC name, we will automatically query the IP under that NIC as the IP to be authorized.

On windows, the NIC name should be like `{93123211-9629-4E04-82F0-EA2E4F221468}`, use `--select-ip` to see.

### Operator selection

Some colleges support network operator selection, which implemented by append the operator code to the username.

Operator code:

- 中国电信: [`chinanet`, `ctcc`] 
- 中国移动: [`cmcc`] 
- 中国联通: [`unicom`, `cucc`]
- 校园网: [`xn`] 

For example, if you choose `cmcc`, just append `@cmcc` to your username, like `202112345@cmcc`.

This code needs to be confirmed by capturing packets.

### TLS support

To keep the binary as small as possible, the pre-compiled binary remove the non-essential `tls` support

If your authentication system uses `https`, You need to compile it yourself with feature `tls` enabled.

```sh
cargo build --features "tls" --release
```

## License

**SDUSrun** © [zu1k](https://github.com/zu1k), Released under the [GPL-3.0](./LICENSE) License.<br>

> Blog [zu1k.com](https://zu1k.com) · GitHub [@zu1k](https://github.com/zu1k) · Twitter [@zu1k_lv](https://twitter.com/zu1k_lv) · Telegram Channel [@peekfun](https://t.me/peekfun)

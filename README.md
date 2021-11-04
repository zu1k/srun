# SDUSRun

[![GitHub stars](https://img.shields.io/github/stars/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/network)
[![GitHub issues](https://img.shields.io/github/issues/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/issues)
[![Build](https://github.com/zu1k/sdusrun/actions/workflows/build.yml/badge.svg)](https://github.com/zu1k/sdusrun/actions/workflows/build.yml)
[![GitHub license](https://img.shields.io/github/license/zu1k/sdusrun)](https://github.com/zu1k/sdusrun/blob/master/LICENSE)

校园网深澜认证登录，SRun 3000

## 使用方法

```
./sdusrun login -u USERNAME -p PASSWORD -i IP [-s AUTH_SERVER_IP]
```

or read config from file

```
./sdusrun login -c config.json
```

config file template

```json
{
    "server": "202.194.15.87",
    "users": [
        {
            "username": "",
            "password": "",
            "ip": ""
        },
        {
            "username": "",
            "password": "",
            "ip": ""
        }
    ]
}
```

## License

GPL-3.0 License

Copyright (c) 2021 zu1k

# redict
A TUI and library to work with DICT servers.

# Compliance

This tool is (for now) partly [RFC 229](https://tools.ietf.org/html/rfc2229) :

- [x] `DEFINE`
- [x] `MATCH`
- [x] `SHOW`
  - [x] `DATABASES`
  - [x] `STRATEGIES`
  - [ ] `INFO`
  - [ ] `SERVER`
- [x] `CLIENT`
- [ ] `STATUS`
- [ ] `QUIT`
- [ ] `OPTION MIME` (and it will never be)
- [ ] `AUTH`
- [ ] SASL

For now, this tool does not read `dict` URLs, either, but will in the future.
It also does not pipelines the requests, as this is not needed in interactive use.

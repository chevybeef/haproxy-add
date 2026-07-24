# Very small, opinionated util for adding entries to haproxy

- cargo run -- /etc/haproxy/haproxy.cfg
- It does NOT parse haproxy.cfg properly - it just finds the last "acl is_" line and the last "use_backend" line inside the frontend block, and inserts new lines right after them. Then it appends a new backend block at the end of the file.
- It never touches the original file: it writes the filename to the current directory for review before manually applying.

```
  frontend Internal
    bind *:443 ssl crt ...
    mode http
    acl is_test hdr(host) -i test.example.com
    ...
    use_backend test_backend if is_test
    ...
  backend test_backend
    mode http
    server test 1.2.3.4:5678 [ssl verify none]
```

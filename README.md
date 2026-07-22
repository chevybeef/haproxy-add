Very small, opinionated helper for a very specific config shape:

  frontend Internal
      bind *:443 ssl crt ...
      mode http
      acl is_X hdr(host) -i X.example.com
      ...
      use_backend X_backend if is_X
      ...
  backend X_backend
      mode http
      server X 1.2.3.4:5678 [ssl verify none]

It does NOT parse haproxy.cfg properly - it just finds the last
"    acl is_" line and the last "    use_backend " line inside the
frontend block, and inserts new lines right after them. Then it
appends a new backend block at the end of the file.

It never touches the original file: it writes "<path>.new" next to
it so you can diff before applying.

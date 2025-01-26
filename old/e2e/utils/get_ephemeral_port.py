import socket

with socket.socket() as s:
  s.bind(('', 0))
  print(s.getsockname()[1], end='')

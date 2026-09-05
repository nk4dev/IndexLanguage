fn fib(n: number): number {
  if n < 2 { return n; }
  return fib(n - 1) + fib(n - 2);
}

let i: number = 0;
while i < 10 {
  print("fib(" + i + ") = " + fib(i));
  i = i + 1;
}

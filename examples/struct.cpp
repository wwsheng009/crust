struct FilePointer {
  int stdin;
  int stdout;
  int stderr;
  char EOF;
};

struct A {
  int a, b, c;
  int a[9],b[9];
  float fa;
};

int main() {
  struct FilePointer fp;
  struct A decl_a;

  // do something with fp here
}

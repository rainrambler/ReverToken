namespace demo {
struct Widget {
    int value;
    int scale(int factor) const { return value * factor; }
};
int add(int a, int b) { return a + b; }
}
extern "C" int plain_c_export(int x) { return x + 1; }

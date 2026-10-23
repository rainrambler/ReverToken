namespace demo {
struct Widget {
    int scale(int value) const { return value * 2; }
};
}
extern "C" int plain_c_export(int x) { return x + 1; }
int demo_add(int a, int b) { return a + b; }

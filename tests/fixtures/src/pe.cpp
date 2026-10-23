namespace demo {
__declspec(dllexport) int add(int a, int b) { return a + b; }
}
extern "C" __declspec(dllexport) int plain_c_export(int x) { return x + 1; }

#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    std::string hello = "Hello from C++~~~~";

     // Get the class for the Kotlin class
     jclass kotlinClass = env->FindClass("com/example/mykotlin/MainActivity");

     // Get the method for the function
     jmethodID functionMethod = env->GetStaticMethodID(kotlinClass, "addOne", "(I)I");

    int x=1;
    int ret=env->CallStaticIntMethod(kotlinClass, functionMethod, x);
    char buf[100];
    sprintf(buf, "Hello from C++~~~~, ret=%d", ret);
    hello=buf;
    return env->NewStringUTF(hello.c_str());
}




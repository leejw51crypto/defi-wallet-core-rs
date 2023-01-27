#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    char buf[100];        
    std::string hello = "Hello from C++~~~~";

     // Get the class for the Kotlin class
     jclass kotlinClass = env->FindClass("com/example/mykotlin/MainActivity");

     // Get the method for the function
     jmethodID functionMethod = env->GetStaticMethodID(kotlinClass, "writeSecureStorage", "(Ljava/lang/String;Ljava/lang/String;)I");

    // make jstring from "key"    
    jstring key=env->NewStringUTF("key2");
    jstring value=env->NewStringUTF("value2");
    jint ret=env->CallStaticIntMethod(kotlinClass, functionMethod, key, value);
    // get int from jint ret
        
    sprintf(buf, "Hello from C++~~~~, ret=%d", ret);
    hello=buf;
    return env->NewStringUTF(hello.c_str());
}

  
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI2(
        JNIEnv* env,
        jobject /* this */) {
    char buf[100];        
    std::string hello = "Hello from C++~~~~";

     // Get the class for the Kotlin class
     jclass kotlinClass = env->FindClass("com/example/mykotlin/MainActivity");

     // Get the method for the function
     jmethodID functionMethod = env->GetStaticMethodID(kotlinClass, "readSecureStorage", "(Ljava/lang/String;)Ljava/util/HashMap;");

    // make jstring from "key"    
    jstring x=env->NewStringUTF("key1");
    jobject ret=env->CallStaticObjectMethod(kotlinClass, functionMethod, x);
    
    // get value from ret
    jstring findkey=env->NewStringUTF("key1");
    jclass mapClass = env->FindClass("java/util/HashMap");
    jmethodID getMethod = env->GetMethodID(mapClass, "get", "(Ljava/lang/Object;)Ljava/lang/Object;");
    jstring value = (jstring)env->CallObjectMethod(ret, getMethod, findkey);
      
    // convert value to string
    const char *nativeString = env->GetStringUTFChars(value, 0);

    
    sprintf(buf, "Hello from C++~~~~, ret=%s", nativeString);
    hello=buf;
    return env->NewStringUTF(hello.c_str());
}



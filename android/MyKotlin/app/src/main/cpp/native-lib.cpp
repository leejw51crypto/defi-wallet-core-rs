#include <jni.h>
#include <string>

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI2(
        JNIEnv* env,
        jobject /* this */) {

    // get application context from env
    jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
    jmethodID currentActivityThreadMethod = env->GetStaticMethodID(activityThreadClass, "currentActivityThread", "()Landroid/app/ActivityThread;");
    jobject activityThread = env->CallStaticObjectMethod(activityThreadClass, currentActivityThreadMethod);
    jmethodID getApplicationMethod = env->GetMethodID(activityThreadClass, "getApplication", "()Landroid/app/Application;");
    jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);

    char buf[100];        
    std::string hello = "Hello from C++~~~~";

     // Get the class for the Kotlin class
     jclass kotlinClass = env->FindClass("com/example/mykotlin/SecureStorage");

     // Get the method for the function
     jmethodID functionMethod = env->GetStaticMethodID(kotlinClass, "writeSecureStorage", "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)I");

    // make jstring from "key"    
    jstring key=env->NewStringUTF("apple");
    jstring value=env->NewStringUTF("computer");
    jint ret=env->CallStaticIntMethod(kotlinClass, functionMethod, context, key, value);
    // get int from jint ret
        
    sprintf(buf, "Hello from C++~~~~, ret=%d", ret);
    hello=buf;
    return env->NewStringUTF(hello.c_str());
}

  
extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI(
        JNIEnv* env,
        jobject /* this */) {
    char buf[100];        
    std::string hello = "Hello from C++~~~~";

    // get application context from env
    jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
    jmethodID currentActivityThreadMethod = env->GetStaticMethodID(activityThreadClass, "currentActivityThread", "()Landroid/app/ActivityThread;");
    jobject activityThread = env->CallStaticObjectMethod(activityThreadClass, currentActivityThreadMethod);
    jmethodID getApplicationMethod = env->GetMethodID(activityThreadClass, "getApplication", "()Landroid/app/Application;");
    jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);



    // Get the class for the Kotlin class
     jclass kotlinClass = env->FindClass("com/example/mykotlin/SecureStorage");

     // Get the method for the function
     jmethodID functionMethod = env->GetStaticMethodID(kotlinClass, "readSecureStorage", "(Landroid/content/Context;sLjava/lang/String;)Ljava/util/HashMap;");

    // make jstring from "key"    
    jstring x=env->NewStringUTF("apple");
    jobject ret=env->CallStaticObjectMethod(kotlinClass, functionMethod,context, x);
    
    // get value from ret
    jstring findkey=env->NewStringUTF("result");
    jclass mapClass = env->FindClass("java/util/HashMap");
    jmethodID getMethod = env->GetMethodID(mapClass, "get", "(Ljava/lang/Object;)Ljava/lang/Object;");
    jstring value = (jstring)env->CallObjectMethod(ret, getMethod, findkey);
      
    // convert value to string
    const char *nativeString = env->GetStringUTFChars(value, 0);

    
    sprintf(buf, "Hello from C++~~~~, ret=%s", nativeString);
    hello=buf;
    return env->NewStringUTF(hello.c_str());
}



#include <jni.h>
#include <string>
using namespace std;
int secureStorageWrite(JNIEnv *env, string userkey, string uservalue) {
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);
  jclass kotlinClass = env->FindClass("com/example/mykotlin/SecureStorage");
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "writeSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)I");
  jstring key = env->NewStringUTF(userkey.c_str());
  jstring value = env->NewStringUTF(uservalue.c_str());
  jint ret = env->CallStaticIntMethod(kotlinClass, functionMethod, context, key,
                                      value);
  return (int)ret;
}

string secureStorageRead(JNIEnv *env, string userkey) {                    
  char buf[100];

  // get application context from env
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);

  // Get the class for the Kotlin class
  jclass kotlinClass = env->FindClass("com/example/mykotlin/SecureStorage");

  // Get the method for the function
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "readSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;)Ljava/util/HashMap;");

  jstring x = env->NewStringUTF(userkey.c_str());
  jobject ret =
      env->CallStaticObjectMethod(kotlinClass, functionMethod, context, x);

  
  jstring resultkey = env->NewStringUTF("result");
  jstring successkey = env->NewStringUTF("success");
  jstring errorkey = env->NewStringUTF("error");
  jclass mapClass = env->FindClass("java/util/HashMap");
  jmethodID getMethod = env->GetMethodID(
      mapClass, "get", "(Ljava/lang/Object;)Ljava/lang/Object;");

  jstring resultvalue = (jstring)env->CallObjectMethod(ret, getMethod, resultkey);
  string resultvaluestring = string(env->GetStringUTFChars(resultvalue, 0));    
  
  jstring successvalue = (jstring)env->CallObjectMethod(ret, getMethod, successkey);
  string successvaluestring = string(env->GetStringUTFChars(successvalue, 0));    

  jstring errorvalue = (jstring)env->CallObjectMethod(ret, getMethod, errorkey);
  string errorvaluestring = string(env->GetStringUTFChars(errorvalue, 0));    

     
  string finalret= resultvaluestring;  
  if ("0"==successvaluestring) { // error
    throw errorvaluestring;
  }

  
  
  return finalret;
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI2(JNIEnv *env,
                                                      jobject /* this */) {

  // get application context from env
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);

  char buf[100];
  std::string hello = "Hello from C++~~~~";

  // Get the class for the Kotlin class
  jclass kotlinClass = env->FindClass("com/example/mykotlin/SecureStorage");

  // Get the method for the function
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "writeSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)I");

  // make jstring from "key"
  jstring key = env->NewStringUTF("apple");
  jstring value = env->NewStringUTF("computer 123~~~~~~~");
  jint ret = env->CallStaticIntMethod(kotlinClass, functionMethod, context, key,
                                      value);
  // get int from jint ret

  sprintf(buf, "Hello from C++~~~~, ret=%d", ret);
  hello = buf;
  return env->NewStringUTF(hello.c_str());
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_mykotlin_MainActivity_stringFromJNI(JNIEnv *env,
                                                     jobject /* this */) {

  try {
    secureStorageWrite(env, "apple", "1234567890");
    string ret=secureStorageRead(env , "apple");
    return env->NewStringUTF(ret.c_str());
  }
  catch(std::string& e) {
    return env->NewStringUTF(e.c_str());
  }
}

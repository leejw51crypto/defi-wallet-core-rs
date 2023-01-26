#include <jni.h>
#include <string>
using namespace std;

#define SECURE_STORAGE_CLASS "com/cronos/play/SecureStorage"

int secureStorageWriteBasic(JNIEnv *env, string userkey, string uservalue) {

  string secureStorageClass = SECURE_STORAGE_CLASS;
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);
  jclass kotlinClass = env->FindClass(secureStorageClass.c_str());
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "writeSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;Ljava/lang/String;)I");
  jstring key = env->NewStringUTF(userkey.c_str());
  jstring value = env->NewStringUTF(uservalue.c_str());
  jint ret = env->CallStaticIntMethod(kotlinClass, functionMethod, context, key,
                                      value);

  return (int)ret;
}

string secureStorageReadBasic(JNIEnv *env, string userkey) {

  string secureStorageClass = SECURE_STORAGE_CLASS;
  jclass activityThreadClass = env->FindClass("android/app/ActivityThread");
  jmethodID currentActivityThreadMethod =
      env->GetStaticMethodID(activityThreadClass, "currentActivityThread",
                             "()Landroid/app/ActivityThread;");
  jobject activityThread = env->CallStaticObjectMethod(
      activityThreadClass, currentActivityThreadMethod);
  jmethodID getApplicationMethod = env->GetMethodID(
      activityThreadClass, "getApplication", "()Landroid/app/Application;");
  jobject context = env->CallObjectMethod(activityThread, getApplicationMethod);
  jclass kotlinClass = env->FindClass(secureStorageClass.c_str());
  jmethodID functionMethod = env->GetStaticMethodID(
      kotlinClass, "readSecureStorage",
      "(Landroid/content/Context;Ljava/lang/String;)Ljava/lang/String;");

  jstring x = env->NewStringUTF(userkey.c_str());
  jobject ret =
      env->CallStaticObjectMethod(kotlinClass, functionMethod, context, x);
  string retstring = string(env->GetStringUTFChars((jstring)ret, 0));

  return retstring;

}

extern "C" JNIEXPORT jstring JNICALL
Java_com_example_myapplication_MainActivity_stringFromJNI(JNIEnv *env,
                                                          jobject /* this */) {

  try {
    // make 64 length string to test
    string s;
    for (int i = 0; i < 64; i++) {
      s += "a";
    }

    secureStorageWriteBasic(env, "apple", "hello world " + s);
    string ret = secureStorageReadBasic(env, "apple");

    return env->NewStringUTF(ret.c_str());
  } catch (std::string &e) {
    return env->NewStringUTF(e.c_str());
  }
}

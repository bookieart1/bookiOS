/*
    SDL_uikit_main.c, placed in the public domain by Sam Lantinga  3/18/2019
*/

/* Include the SDL main definition header */
#include "SDL_main.h"

#if defined(__IPHONEOS__) || defined(__TVOS__)

#ifndef SDL_MAIN_HANDLED
#ifdef main
#undef main
#endif


/* The native app picker: Swift in AppPicker.swift, given to Rust here. */
#include <stddef.h>
#include <stdint.h>
typedef char *(*idared_app_picker_fn)(const char *, const uint8_t *const *, size_t);
extern char *idared_swiftui_app_picker(const char *, const uint8_t *const *, size_t);
extern void idared_set_swiftui_app_picker(idared_app_picker_fn);
typedef char *(*idared_text_alert_fn)(const char *);
extern char *idared_native_text_alert(const char *);
extern void idared_set_native_text_alert(idared_text_alert_fn);
/* Gives games opened with "Open in iDared 32bit" to the picker too. */
extern void idared_install_open_in_hook(void);
/* The guest selects an interface orientation before its window is shown. */
typedef void (*idared_host_orientation_fn)(int32_t);
extern void idared_request_host_orientation(int32_t);
extern void idared_set_host_orientation(idared_host_orientation_fn);
/* Explicit guest/picker ownership invalidates delayed guest rotations. */
typedef void (*idared_host_presentation_fn)(int32_t);
extern void idared_set_host_presentation_owner(int32_t);
extern void idared_set_host_presentation(idared_host_presentation_fn);
/* Bounded native presentation events are routed into Rust's device Log.txt. */
typedef void (*idared_host_diagnostic_fn)(const char *, const char *);
extern void idared_native_presentation_diagnostic_event(const char *, const char *);
extern void idared_set_host_diagnostic(idared_host_diagnostic_fn);

int
main(int argc, char *argv[])
{
    idared_set_swiftui_app_picker(idared_swiftui_app_picker);
    idared_set_native_text_alert(idared_native_text_alert);
    idared_set_host_orientation(idared_request_host_orientation);
    idared_set_host_presentation(idared_set_host_presentation_owner);
    idared_set_host_diagnostic(idared_native_presentation_diagnostic_event);
    idared_install_open_in_hook();
    return SDL_UIKitRunApp(argc, argv, SDL_main);
}
#endif /* !SDL_MAIN_HANDLED */

#endif /* __IPHONEOS__ || __TVOS__ */

/* vi: set ts=4 sw=4 expandtab: */

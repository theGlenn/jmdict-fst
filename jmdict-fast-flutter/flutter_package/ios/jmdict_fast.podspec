#
# Podspec for the iOS slice of the jmdict_fast Flutter plugin.
# Run `pod lib lint jmdict_fast.podspec` to validate before publishing.
#
Pod::Spec.new do |s|
  s.name             = 'jmdict_fast'
  s.version          = '0.1.0'
  s.summary          = 'Blazing-fast Japanese dictionary engine for Flutter (FRB).'
  s.description      = <<-DESC
JMdict-based Japanese dictionary with FST-indexed lookup, gloss reverse
search, deinflection, and a one-call install path. Backed by the
jmdict-fast Rust engine via flutter_rust_bridge.
                       DESC
  s.homepage         = 'https://github.com/theGlenn/jmdict-fst'
  s.license          = { :file => '../LICENSE' }
  s.author           = { 'Glenn Sonna' => 'theGlenn@users.noreply.github.com' }

  s.source           = { :path => '.' }
  s.source_files     = 'Classes/**/*'
  s.dependency 'Flutter'
  s.platform = :ios, '11.0'

  s.pod_target_xcconfig = { 'DEFINES_MODULE' => 'YES', 'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386' }
  s.swift_version = '5.0'

  s.script_phase = {
    :name => 'Build Rust library',
    # Args: (1) path to the cargo crate, (2) cargo lib name. The binding
    # crate sits two levels up from `flutter_package/ios/`.
    :script => 'sh "$PODS_TARGET_SRCROOT/../cargokit/build_pod.sh" ../../ jmdict_fast_flutter',
    :execution_position => :before_compile,
    :input_files => ['${BUILT_PRODUCTS_DIR}/cargokit_phony'],
    :output_files => ["${BUILT_PRODUCTS_DIR}/libjmdict_fast_flutter.a"],
  }
  s.pod_target_xcconfig = {
    'DEFINES_MODULE' => 'YES',
    'EXCLUDED_ARCHS[sdk=iphonesimulator*]' => 'i386',
    'OTHER_LDFLAGS' => '-force_load ${BUILT_PRODUCTS_DIR}/libjmdict_fast_flutter.a',
  }
end

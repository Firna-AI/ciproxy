#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the template and toolset preparation offline, without a Mac or Packer."""

import importlib.util
import json
import pathlib
import unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("prepare", HERE / "prepare-template.py")
prepare = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prepare)

TEMPLATE = '''packer {
  required_plugins {
    veertu-anka = {
      version = ">= v3.2.0"
    }
  }
}

variable "builder_type" {
  type = string
  default = "veertu-anka-vm-clone"
}

source "veertu-anka-vm-clone" "template" {
  vm_name = "${var.build_id}"
}

source "null" "template" {
  ssh_host = "${var.source_vm_name}"
  ssh_password = "${var.vm_password}"
}

build {
  sources = ["source.${var.builder_type}.template"]
  provisioner "file" {
    source = "${path.root}/../toolsets/toolset-26.json"
  }
  provisioner "shell" {
    scripts = ["install.sh"]
  }
  provisioner "shell" {
    expect_disconnect = true
    inline = ["shutdown -r now"]
  }
}
'''


def toolset(default_runtimes="none"):
    versions = [{"link": "26.6", "version": "26.6+17F113", "sha256": "aa",
                 "install_runtimes": default_runtimes},
                {"link": "26.5", "version": "26.5+17F42", "sha256": "bb",
                 "install_runtimes": "default"}]
    return json.dumps({"xcode": {"default": "26.6", "arm64": {"versions": versions}},
                       "brew": {}})


class Template(unittest.TestCase):
    def test_null_builder_and_ciproxy_steps(self):
        text = prepare.prepare_template(TEMPLATE)
        self.assertNotIn("packer {", text)
        self.assertNotIn('source "veertu-anka-vm-clone"', text)
        self.assertIn('default = "null"', text)
        self.assertIn('source "null" "template"', text)
        self.assertIn(prepare.TOOLSET_NAME, text)
        self.assertIn("./install.sh ${var.build_id}", text)
        self.assertIn('variable "ciproxy_inputs"', text)
        self.assertLess(text.index("install-runner.sh"), text.index('variable "ciproxy_inputs"'))
        self.assertEqual(text.count("{"), text.count("}"))

    def test_key_login_and_retries(self):
        text = prepare.prepare_template(TEMPLATE)
        self.assertNotIn("ssh_password", text)
        self.assertIn('ssh_private_key_file = "${var.ciproxy_ssh_key}"', text)
        self.assertIn('variable "ciproxy_ssh_key"', text)
        install = text.index('scripts = ["install.sh"]')
        restart = text.index("expect_disconnect = true")
        self.assertIn("max_retries = 2", text[text.rindex('provisioner "shell"', 0, install):install])
        self.assertNotIn("max_retries", text[text.rindex('provisioner "shell"', 0, restart):restart])

    def test_changed_upstream_fails(self):
        with self.assertRaises(SystemExit):
            prepare.prepare_template(TEMPLATE.replace("veertu-anka-vm-clone", "other"))


class Toolset(unittest.TestCase):
    def test_default_only_installs_its_runtimes(self):
        result = json.loads(prepare.prepare_toolset(toolset(), ["26.6"]))
        versions = result["xcode"]["arm64"]["versions"]
        self.assertEqual([entry["link"] for entry in versions], ["26.6"])
        self.assertEqual(versions[0]["install_runtimes"], "default")
        self.assertIn("brew", result)

    def test_full_set_keeps_upstream_runtimes(self):
        result = json.loads(prepare.prepare_toolset(toolset(), ["26.6", "26.5"]))
        runtimes = {entry["link"]: entry["install_runtimes"]
                    for entry in result["xcode"]["arm64"]["versions"]}
        self.assertEqual(runtimes, {"26.6": "none", "26.5": "default"})

    def test_missing_version_or_default_fails(self):
        with self.assertRaises(SystemExit):
            prepare.prepare_toolset(toolset(), ["26.6", "25.0"])
        with self.assertRaises(SystemExit):
            prepare.prepare_toolset(toolset(), ["26.5"])

    def test_xcode_pin_must_match_toolset(self):
        prepared = prepare.prepare_toolset(toolset(), ["26.6"])
        prepare.check_xcode_pin(prepared, "Xcode-26.6+17F113.xip", "aa")
        with self.assertRaises(SystemExit):
            prepare.check_xcode_pin(prepared, "Xcode-26.6+17F113.xip", "bb")


if __name__ == "__main__":
    unittest.main()

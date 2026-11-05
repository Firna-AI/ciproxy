#!/usr/bin/env python3
"""Operator setup for one namespace. CI cannot create namespaces or change RBAC."""

import argparse
import json
from pathlib import Path
import subprocess


def configuration(environment):
    namespace = f"ciproxy-{environment}"
    metadata = {"name": "ciproxy-deploy", "namespace": namespace}
    return {"apiVersion": "v1", "kind": "List", "items": [
        {"apiVersion": "v1", "kind": "Namespace", "metadata": {"name": namespace, "labels": {
            "pod-security.kubernetes.io/enforce": "baseline",
            "pod-security.kubernetes.io/audit": "baseline",
            "pod-security.kubernetes.io/warn": "baseline",
        }}},
        {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "Role", "metadata": metadata,
         "rules": [
             {"apiGroups": [""], "resources": ["secrets", "services", "serviceaccounts", "configmaps"],
              "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]},
             {"apiGroups": [""], "resources": ["pods", "pods/log", "events"],
              "verbs": ["get", "list", "watch"]},
             {"apiGroups": ["apps"], "resources": ["deployments", "replicasets"],
              "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]},
             {"apiGroups": ["secrets-store.csi.x-k8s.io"], "resources": ["secretproviderclasses"],
              "verbs": ["get", "list", "watch", "create", "update", "patch", "delete"]},
         ]},
        {"apiVersion": "rbac.authorization.k8s.io/v1", "kind": "RoleBinding", "metadata": metadata,
         "subjects": [{"kind": "User", "name": f"ciproxy-{environment}-deploy@ciproxy.iam.gserviceaccount.com"}],
         "roleRef": {"apiGroup": "rbac.authorization.k8s.io", "kind": "Role", "name": "ciproxy-deploy"}},
        {"apiVersion": "networking.k8s.io/v1", "kind": "NetworkPolicy",
         "metadata": {"name": "environment-boundary", "namespace": namespace},
         "spec": {"podSelector": {}, "policyTypes": ["Ingress", "Egress"],
                  "ingress": [{"from": [{"podSelector": {}},
                                         {"ipBlock": {"cidr": "35.191.0.0/16"}},
                                         {"ipBlock": {"cidr": "130.211.0.0/22"}}]}],
                  "egress": [{"to": [{"podSelector": {}}]},
                             {"to": [{"ipBlock": {"cidr": "0.0.0.0/0", "except": ["10.84.0.0/14"]}}]},
                             {"to": [{"namespaceSelector": {"matchLabels": {"kubernetes.io/metadata.name": "kube-system"}}}],
                              "ports": [{"protocol": "UDP", "port": 53}, {"protocol": "TCP", "port": 53}]}]}},
        {"apiVersion": "v1", "kind": "ServiceAccount",
         "metadata": {"name": "control-plane", "namespace": namespace, "annotations": {
             "iam.gke.io/gcp-service-account": f"ciproxy-{environment}-control@ciproxy.iam.gserviceaccount.com",
         }}},
    ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("--render", action="store_true")
    parser.add_argument("--uid-file", type=Path, help="Save the real operator-created KSA UID for trusted bootstrap.")
    args = parser.parse_args()
    document = json.dumps(configuration(args.environment))
    if args.render:
        print(document)
        return
    subprocess.run(["gcloud", "container", "clusters", "get-credentials", "ciproxy-shared",
                    "--region=europe-west1", "--project=ciproxy", "--dns-endpoint"], check=True)
    subprocess.run(["kubectl", "apply", "--server-side", "--field-manager=ciproxy-operator", "-f", "-"],
                   check=True, input=document, text=True)
    if args.uid_file:
        uid = subprocess.check_output(["kubectl", "get", "serviceaccount", "control-plane",
                                       f"--namespace=ciproxy-{args.environment}", "-o", "jsonpath={.metadata.uid}"], text=True).strip()
        if not uid:
            raise ValueError("The intended cluster returned no ServiceAccount UID. No trust input was written.")
        inputs = json.loads(args.uid_file.read_text()) if args.uid_file.exists() else {"runtime_service_account_uids": {}}
        inputs["runtime_service_account_uids"][args.environment] = uid
        args.uid_file.parent.mkdir(parents=True, exist_ok=True)
        args.uid_file.write_text(json.dumps(inputs, indent=2) + "\n")
        print(f"Captured {args.environment} runtime UID in {args.uid_file}. Apply it through trusted bootstrap.")


if __name__ == "__main__":
    main()

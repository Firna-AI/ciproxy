#!/usr/bin/env python3
"""Operator setup for one namespace. CI cannot create namespaces or change RBAC."""

import argparse
import json
import subprocess


def configuration(environment):
    namespace = f"ciproxy-{environment}"
    metadata = {"name": "ciproxy-deploy", "namespace": namespace}
    return {"apiVersion": "v1", "kind": "List", "items": [
        {"apiVersion": "v1", "kind": "Namespace", "metadata": {"name": namespace}},
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
    ]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("environment", choices=["main-preview", "production"])
    parser.add_argument("--render", action="store_true")
    args = parser.parse_args()
    document = json.dumps(configuration(args.environment))
    if args.render:
        print(document)
        return
    subprocess.run(["gcloud", "container", "clusters", "get-credentials", "ciproxy-shared",
                    "--region=europe-west1", "--project=ciproxy", "--dns-endpoint"], check=True)
    subprocess.run(["kubectl", "apply", "--server-side", "--field-manager=ciproxy-operator", "-f", "-"],
                   check=True, input=document, text=True)


if __name__ == "__main__":
    main()

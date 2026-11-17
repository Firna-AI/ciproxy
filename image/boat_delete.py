# SPDX-License-Identifier: Apache-2.0
"""Send Boat sandbox deletion with confirmation of its exact target."""

from urllib.parse import quote


def confirmed_delete(request, sandbox, **kwargs):
    return request("DELETE", "/sandboxes/" + quote(sandbox, safe=""),
                   headers={"X-Ascii-Confirm-Delete": sandbox}, **kwargs)

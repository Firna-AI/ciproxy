// The line of each checkout review with a pay button: paid plans are for
// business use only, as the terms say. A review that cannot pay shows none.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";

export function BusinessUseNote() {
  return (
    <p className="caption">
      Paid plans are for business use only, as the ciproxy{" "}
      <TextLink to={paths.terms} newTab>
        Terms
      </TextLink>{" "}
      say.
    </p>
  );
}

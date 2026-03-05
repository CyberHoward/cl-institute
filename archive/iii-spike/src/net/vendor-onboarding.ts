import type { Net } from "./types.js";

export const vendorOnboardingNet: Net = {
  id: "vendor-onboarding",
  places: [
    {
      id: "request-submitted",
      description: "A vendor onboarding request has been submitted with vendor details",
    },
    {
      id: "vendor-verified",
      description:
        "The vendor's identity and registration have been confirmed as legitimate",
    },
    {
      id: "risk-assessed",
      description:
        "A risk assessment has been completed with a risk level and supporting rationale",
    },
    {
      id: "compliance-notified",
      description:
        "The compliance team has been notified of the vendor assessment and required actions",
    },
    {
      id: "onboarding-approved",
      description:
        "The onboarding has been approved (or rejected) with a documented decision",
    },
  ],
  transitions: [
    {
      id: "verify-vendor",
      consumes: ["request-submitted"],
      produces: ["vendor-verified"],
      intent:
        "Verify that the vendor exists as a registered entity, confirm their business " +
        "registration is active, and retrieve their basic profile including contact " +
        "information, certifications, and compliance history. This is a factual " +
        "verification step — no judgment is required, just confirm the data.",
      context_sources: ["vendor-request"],
      postconditions: {
        required: [
          "vendor-identity-confirmed",
          "vendor-profile-retrieved",
        ],
        desired: [
          "vendor-certifications-listed",
          "vendor-compliance-history-available",
        ],
      },
      available_tools: ["lookup-vendor"],
      mode: "deterministic",
    },
    {
      id: "assess-risk",
      consumes: ["vendor-verified"],
      produces: ["risk-assessed"],
      intent:
        "Assess the risk level of onboarding this vendor by reviewing their profile " +
        "data against the organization's risk policy. Consider the vendor's " +
        "certifications, compliance history, jurisdiction, contract history, and " +
        "any risk flags. Produce a risk assessment document that classifies the " +
        "vendor as low, medium, or high risk with supporting rationale.",
      context_sources: ["vendor-profile", "risk-policy"],
      postconditions: {
        required: [
          "risk-level-determined",
          "risk-assessment-documented",
        ],
        desired: [
          "specific-risk-factors-identified",
          "recommended-actions-listed",
        ],
      },
      available_tools: ["generate-document"],
      mode: "judgment",
    },
    {
      id: "notify-compliance",
      consumes: ["risk-assessed"],
      produces: ["compliance-notified"],
      intent:
        "Notify the compliance team about this vendor's risk assessment results. " +
        "The notification should include the vendor name, risk level, key risk " +
        "factors, and any recommended actions. Use the appropriate notification " +
        "channel based on the risk level: email for low risk, Slack for medium " +
        "or high risk (for faster response).",
      context_sources: ["vendor-profile", "risk-assessment"],
      postconditions: {
        required: ["notification-sent"],
        desired: ["notification-includes-risk-summary"],
      },
      available_tools: ["send-notification"],
      mode: "agentic",
    },
    {
      id: "approve-onboarding",
      consumes: ["compliance-notified"],
      produces: ["onboarding-approved"],
      intent:
        "Review all gathered information — vendor profile, risk assessment, and " +
        "compliance notification status — and make an onboarding recommendation. " +
        "For low-risk vendors, recommend approval. For medium-risk vendors, " +
        "recommend conditional approval with specific conditions. For high-risk " +
        "vendors, recommend further review or rejection. Document the decision " +
        "with clear rationale.",
      context_sources: [
        "vendor-profile",
        "risk-assessment",
        "compliance-notification",
      ],
      postconditions: {
        required: [
          "onboarding-decision-made",
          "decision-rationale-documented",
        ],
        desired: ["conditions-specified-if-applicable"],
        escalation: ["escalate-to-senior-compliance"],
      },
      available_tools: ["generate-document"],
      mode: "judgment",
    },
  ],
};

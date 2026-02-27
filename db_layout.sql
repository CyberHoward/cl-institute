-- ============================================================
-- LAYER 1: MULTI-ORG & AUTH
-- ============================================================

-- STEP 1: Create all tables and types first (no policies)
-- ============================================================

-- Extends Supabase auth.users with app-specific profile data
CREATE TABLE public.profiles (
  id           UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  display_name TEXT NOT NULL,
  avatar_url   TEXT,
  created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE public.organizations (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name        TEXT NOT NULL,
  description TEXT,
  rules       JSONB NOT NULL DEFAULT '{}'::jsonb,  -- Record<string, string>: named lists of text rules (key: rule name, value: rule description)
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Organization membership (for access control & permissions)
CREATE TYPE public.organization_permission_level AS ENUM ('owner', 'admin', 'member', 'viewer');

CREATE TABLE public.organization_members (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id  UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
  user_id          UUID NOT NULL REFERENCES public.profiles(id) ON DELETE CASCADE,
  permission_level public.organization_permission_level NOT NULL DEFAULT 'member',
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, user_id)
);

-- ============================================================
-- LAYER 2: ORGANIZATION MODEL
-- ============================================================

CREATE TABLE public.organizational_roles (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
  name            TEXT NOT NULL,
  description     TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, name)
);

-- Assign institutional roles to organization members
CREATE TABLE public.member_organizational_roles (
  id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_member_id  UUID NOT NULL REFERENCES public.organization_members(id) ON DELETE CASCADE,
  organizational_role_id  UUID NOT NULL REFERENCES public.organizational_roles(id) ON DELETE CASCADE,
  created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (organization_member_id, organizational_role_id)
);

-- Functions group related workflows (protocols) within an organization
CREATE TABLE public.functions (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
  name            TEXT NOT NULL,
  description     TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (organization_id, name)
);

-- ============================================================
-- LAYER 3: WORKFLOW DEFINITION
-- ============================================================

CREATE TABLE public.workflows (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID NOT NULL REFERENCES public.organizations(id) ON DELETE CASCADE,
  function_id     UUID REFERENCES public.functions(id) ON DELETE SET NULL,
  name            TEXT NOT NULL,
  description     TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TYPE public.node_type AS ENUM ('start', 'intermediate', 'end');

CREATE TABLE public.nodes (
  id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workflow_id UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  type        public.node_type NOT NULL,
  label       TEXT NOT NULL,
  index  INTEGER NOT NULL DEFAULT 0,
  x           FLOAT NOT NULL DEFAULT 0,
  y           FLOAT NOT NULL DEFAULT 0,
  created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE public.edges (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workflow_id   UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  from_node_id  UUID NOT NULL REFERENCES public.nodes(id) ON DELETE CASCADE,
  to_node_id    UUID NOT NULL REFERENCES public.nodes(id) ON DELETE CASCADE,
  label         TEXT,
  rule          TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (workflow_id, from_node_id, to_node_id)
);

CREATE TYPE public.requirement_type AS ENUM ('document', 'approval');

CREATE TABLE public.edge_requirements (
  id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  edge_id          UUID NOT NULL REFERENCES public.edges(id) ON DELETE CASCADE,
  type             public.requirement_type NOT NULL,
  label            TEXT NOT NULL,
  description      TEXT,
  config           JSONB,
  is_optional      BOOLEAN NOT NULL DEFAULT FALSE,
  created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tracks version history for workflows (JSONB snapshot of nodes + edges)
CREATE TABLE public.workflow_versions (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workflow_id   UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  version       INTEGER NOT NULL,
  change_reason TEXT NOT NULL,
  snapshot      JSONB NOT NULL,
  created_by    UUID NOT NULL REFERENCES public.profiles(id),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (workflow_id, version)
);

-- Define which organizational roles can execute a transition (edge)
CREATE TABLE public.edge_role_permissions (
  id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  edge_id                UUID NOT NULL REFERENCES public.edges(id) ON DELETE CASCADE,
  organizational_role_id UUID NOT NULL REFERENCES public.organizational_roles(id) ON DELETE CASCADE,
  created_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at             TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  UNIQUE (edge_id, organizational_role_id)
);

-- ============================================================
-- LAYER 4: WORKFLOW EXECUTION
-- ============================================================

CREATE TYPE public.instance_status AS ENUM ('IN_PROGRESS', 'COMPLETED', 'REJECTED', 'CANCELLED');

CREATE TABLE public.workflow_instances (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  workflow_id     UUID NOT NULL REFERENCES public.workflows(id) ON DELETE CASCADE,
  applicant_name  TEXT NOT NULL,
  current_node_id UUID REFERENCES public.nodes(id) ON DELETE SET NULL,
  status          public.instance_status NOT NULL DEFAULT 'IN_PROGRESS',
  trace           JSONB NOT NULL DEFAULT '[]',
  created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE public.document_submissions (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  edge_requirement_id  UUID NOT NULL REFERENCES public.edge_requirements(id) ON DELETE CASCADE,
  workflow_instance_id UUID NOT NULL REFERENCES public.workflow_instances(id) ON DELETE CASCADE,
  submitted_by_user_id UUID NOT NULL REFERENCES public.profiles(id) ON DELETE CASCADE,
  file_name            TEXT NOT NULL,
  file_path            TEXT NOT NULL,
  file_size_bytes      BIGINT NOT NULL,
  file_type            TEXT NOT NULL,
  notes                TEXT,
  submitted_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================
-- STEP 2: Create indexes
-- ============================================================

-- Indexes for profiles
CREATE INDEX idx_profiles_display_name ON public.profiles(display_name);

-- Indexes for organizations
CREATE INDEX idx_organizations_name ON public.organizations(name);

-- Indexes for organization_members
CREATE INDEX idx_org_members_org_id ON public.organization_members(organization_id);
CREATE INDEX idx_org_members_user_id ON public.organization_members(user_id);
CREATE INDEX idx_org_members_permission_level ON public.organization_members(permission_level);

-- Indexes for organizational_roles
CREATE INDEX idx_org_roles_org_id ON public.organizational_roles(organization_id);

-- Indexes for member_organizational_roles
CREATE INDEX idx_member_org_roles_member_id ON public.member_organizational_roles(organization_member_id);
CREATE INDEX idx_member_org_roles_role_id ON public.member_organizational_roles(organizational_role_id);

-- Indexes for functions
CREATE INDEX idx_functions_org_id ON public.functions(organization_id);

-- Indexes for workflows
CREATE INDEX idx_workflows_org_id ON public.workflows(organization_id);
CREATE INDEX idx_workflows_function_id ON public.workflows(function_id);

-- Indexes for nodes
CREATE INDEX idx_nodes_workflow_id ON public.nodes(workflow_id);

-- Indexes for edges
CREATE INDEX idx_edges_workflow_id ON public.edges(workflow_id);
CREATE INDEX idx_edges_from_node ON public.edges(from_node_id);
CREATE INDEX idx_edges_to_node ON public.edges(to_node_id);

-- Indexes for workflow_versions
CREATE INDEX idx_workflow_versions_workflow_id ON public.workflow_versions(workflow_id);
CREATE INDEX idx_workflow_versions_created_by ON public.workflow_versions(created_by);

-- Indexes for edge_requirements
CREATE INDEX idx_edge_requirements_edge_id ON public.edge_requirements(edge_id);

-- Indexes for edge_role_permissions
CREATE INDEX idx_edge_role_perms_edge_id ON public.edge_role_permissions(edge_id);
CREATE INDEX idx_edge_role_perms_role_id ON public.edge_role_permissions(organizational_role_id);

-- Indexes for workflow_instances
CREATE INDEX idx_workflow_instances_workflow_id ON public.workflow_instances(workflow_id);
CREATE INDEX idx_workflow_instances_current_node ON public.workflow_instances(current_node_id);
CREATE INDEX idx_workflow_instances_status ON public.workflow_instances(status);
CREATE INDEX idx_workflow_instances_created_at ON public.workflow_instances(created_at DESC);

-- Indexes for document_submissions
CREATE INDEX idx_document_submissions_instance ON public.document_submissions(workflow_instance_id);
CREATE INDEX idx_document_submissions_requirement ON public.document_submissions(edge_requirement_id);
CREATE INDEX idx_document_submissions_user ON public.document_submissions(submitted_by_user_id);
CREATE INDEX idx_document_submissions_submitted_at ON public.document_submissions(submitted_at DESC);

-- ============================================================
-- STEP 3: Enable RLS on all tables
-- ============================================================

ALTER TABLE public.profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.organization_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.organizational_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.member_organizational_roles ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.functions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.workflows ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.nodes ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.edges ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.edge_requirements ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.edge_role_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.workflow_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.workflow_instances ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.document_submissions ENABLE ROW LEVEL SECURITY;

-- ============================================================
-- STEP 4: Create RLS policies (now all tables exist)
-- ============================================================

-- RLS Policies for profiles
CREATE POLICY "Users can view all profiles"
  ON public.profiles FOR SELECT
  USING (true);

CREATE POLICY "Users can update their own profile"
  ON public.profiles FOR UPDATE
  USING (auth.uid() = id);

CREATE POLICY "Users can insert their own profile"
  ON public.profiles FOR INSERT
  WITH CHECK (auth.uid() = id);

-- RLS Policies for organizations
CREATE POLICY "Users can view organizations they are members of"
  ON public.organizations FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = organizations.id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization owners can update their organization"
  ON public.organizations FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = organizations.id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level = 'owner'
    )
  );

CREATE POLICY "Authenticated users can create organizations"
  ON public.organizations FOR INSERT
  WITH CHECK (auth.uid() IS NOT NULL);

-- RLS Policies for organization_members
CREATE POLICY "Users can view members of their organizations"
  ON public.organization_members FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members om
      WHERE om.organization_id = organization_members.organization_id
        AND om.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage members"
  ON public.organization_members FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members om
      WHERE om.organization_id = organization_members.organization_id
        AND om.user_id = auth.uid()
        AND om.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for organizational_roles
CREATE POLICY "Users can view roles in their organizations"
  ON public.organizational_roles FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = organizational_roles.organization_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage roles"
  ON public.organizational_roles FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = organizational_roles.organization_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for member_organizational_roles
CREATE POLICY "Users can view role assignments in their organizations"
  ON public.member_organizational_roles FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members om1
      JOIN public.organization_members om2 ON om1.organization_id = om2.organization_id
      WHERE om1.id = member_organizational_roles.organization_member_id
        AND om2.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage role assignments"
  ON public.member_organizational_roles FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members om1
      JOIN public.organization_members om2 ON om1.organization_id = om2.organization_id
      WHERE om1.id = member_organizational_roles.organization_member_id
        AND om2.user_id = auth.uid()
        AND om2.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for functions
CREATE POLICY "Users can view functions in their organizations"
  ON public.functions FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = functions.organization_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage functions"
  ON public.functions FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = functions.organization_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for workflows
CREATE POLICY "Users can view workflows in their organizations"
  ON public.workflows FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = workflows.organization_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage workflows"
  ON public.workflows FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.organization_members
      WHERE organization_members.organization_id = workflows.organization_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for nodes
CREATE POLICY "Users can view nodes in workflows they have access to"
  ON public.nodes FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = nodes.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage nodes"
  ON public.nodes FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = nodes.workflow_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for edges
CREATE POLICY "Users can view edges in workflows they have access to"
  ON public.edges FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = edges.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage edges"
  ON public.edges FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = edges.workflow_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for edge_requirements
CREATE POLICY "Users can view edge requirements in workflows they have access to"
  ON public.edge_requirements FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.edges
      JOIN public.workflows ON workflows.id = edges.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE edges.id = edge_requirements.edge_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage edge requirements"
  ON public.edge_requirements FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.edges
      JOIN public.workflows ON workflows.id = edges.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE edges.id = edge_requirements.edge_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for edge_role_permissions
CREATE POLICY "Users can view edge role permissions in workflows they have access to"
  ON public.edge_role_permissions FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.edges
      JOIN public.workflows ON workflows.id = edges.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE edges.id = edge_role_permissions.edge_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage edge role permissions"
  ON public.edge_role_permissions FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.edges
      JOIN public.workflows ON workflows.id = edges.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE edges.id = edge_role_permissions.edge_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for workflow_versions
CREATE POLICY "Users can view workflow versions in their organizations"
  ON public.workflow_versions FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = workflow_versions.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization admins and owners can manage workflow versions"
  ON public.workflow_versions FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = workflow_versions.workflow_id
        AND organization_members.user_id = auth.uid()
        AND organization_members.permission_level IN ('owner', 'admin')
    )
  );

-- RLS Policies for workflow_instances
CREATE POLICY "Users can view workflow instances in their organizations"
  ON public.workflow_instances FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = workflow_instances.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization members can create workflow instances"
  ON public.workflow_instances FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = workflow_instances.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization members can update workflow instances"
  ON public.workflow_instances FOR UPDATE
  USING (
    EXISTS (
      SELECT 1 FROM public.workflows
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflows.id = workflow_instances.workflow_id
        AND organization_members.user_id = auth.uid()
    )
  );

-- RLS Policies for document_submissions
CREATE POLICY "Users can view document submissions in their organizations"
  ON public.document_submissions FOR SELECT
  USING (
    EXISTS (
      SELECT 1 FROM public.workflow_instances
      JOIN public.workflows ON workflows.id = workflow_instances.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflow_instances.id = document_submissions.workflow_instance_id
        AND organization_members.user_id = auth.uid()
    )
  );

CREATE POLICY "Organization members can submit documents"
  ON public.document_submissions FOR INSERT
  WITH CHECK (
    EXISTS (
      SELECT 1 FROM public.workflow_instances
      JOIN public.workflows ON workflows.id = workflow_instances.workflow_id
      JOIN public.organization_members ON organization_members.organization_id = workflows.organization_id
      WHERE workflow_instances.id = document_submissions.workflow_instance_id
        AND organization_members.user_id = auth.uid()
    )
    AND submitted_by_user_id = auth.uid()
  );

-- ============================================================
-- STEP 5: Create triggers for updated_at
-- ============================================================

CREATE OR REPLACE FUNCTION public.update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers to all tables with updated_at
CREATE TRIGGER update_profiles_updated_at BEFORE UPDATE ON public.profiles
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_organizations_updated_at BEFORE UPDATE ON public.organizations
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_organization_members_updated_at BEFORE UPDATE ON public.organization_members
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_organizational_roles_updated_at BEFORE UPDATE ON public.organizational_roles
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_member_organizational_roles_updated_at BEFORE UPDATE ON public.member_organizational_roles
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_functions_updated_at BEFORE UPDATE ON public.functions
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_workflows_updated_at BEFORE UPDATE ON public.workflows
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_nodes_updated_at BEFORE UPDATE ON public.nodes
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_edges_updated_at BEFORE UPDATE ON public.edges
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_edge_requirements_updated_at BEFORE UPDATE ON public.edge_requirements
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_edge_role_permissions_updated_at BEFORE UPDATE ON public.edge_role_permissions
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();

CREATE TRIGGER update_workflow_instances_updated_at BEFORE UPDATE ON public.workflow_instances
  FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();